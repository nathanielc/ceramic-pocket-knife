use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use futures::stream::{self, Stream, StreamExt};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, sleep_until, Duration, Instant, Sleep};
use uuid::Uuid;

use ceramic_core::{Cid, DidDocument, JwkSigner, Jws};

use crate::{
    ceramic,
    cli::{CasAnchorRequestArgs, CasLoadGenArgs, Command},
};

pub enum Operation {
    AnchorRequest(CasAnchorRequestArgs),
    LoadGen(CasLoadGenArgs),
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::CasAnchorRequest(args) => Ok(Operation::AnchorRequest(args)),
            Command::CasLoadGen(args) => Ok(Operation::LoadGen(args)),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation) -> Result<()> {
    match op {
        Operation::AnchorRequest(args) => {
            anchor_request(args).await?;
        }
        Operation::LoadGen(args) => {
            load_gen(&AtomicU64::new(1), args).await?;
        }
    };
    Ok(())
}

async fn anchor_request(args: CasAnchorRequestArgs) -> Result<()> {
    // Create a new stream and its corresponding anchor request CAR bytes
    let (root_cid, car_bytes) =
        ceramic::create_stream_car(args.r#type, args.stream_controller, args.unique).await?;
    // Send the anchor request
    let cas_url = format!("{}/api/v0/requests", args.url);
    let auth_header = auth_header(cas_url.clone(), args.node_controller, root_cid).await?;
    let res = reqwest::Client::new()
        .post(cas_url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/vnd.ipld.car")
        .body(car_bytes)
        .send()
        .await?;
    println!("{}", res.text().await?);
    Ok(())
}

/// Generate a JWS-signed header for use with CAS Auth
async fn auth_header(url: String, controller: String, digest: Cid) -> Result<String> {
    #[derive(Serialize, Deserialize)]
    struct CasAuthPayload {
        url: String,
        nonce: String,
        digest: Cid,
    }
    let auth_payload = CasAuthPayload {
        url,
        nonce: Uuid::new_v4().to_string(),
        digest,
    };
    // Retrieve the node private key from the environment, if available, otherwise generate a random private key.
    let node_private_key = std::env::var("NODE_PRIVATE_KEY").unwrap_or_else(|_| random_secret(32));
    let signer = JwkSigner::new(
        DidDocument::new(controller.as_str()),
        node_private_key.as_str(),
    )
    .await
    .unwrap();
    let auth_jws = Jws::for_data(&signer, &auth_payload).await?;
    let (sig, protected) = auth_jws
        .signatures
        .first()
        .and_then(|sig| sig.protected.as_ref().map(|p| (&sig.signature, p)))
        .unwrap();
    Ok(format!("Bearer {}.{}.{}", protected, auth_jws.payload, sig))
}

/// Generate a random, hex-encoded secret
pub fn random_secret(len: usize) -> String {
    let mut data = vec![0u8; len];
    thread_rng().fill_bytes(&mut data[..]);
    hex::encode(data)
}

async fn load_gen(counter: &AtomicU64, args: CasLoadGenArgs) -> Result<()> {
    if (args.count == 0) || (args.rate == Some(0)) {
        return Ok(());
    }
    let anchor_request_args = CasAnchorRequestArgs {
        url: args.url,
        node_controller: args.node_controller,
        r#type: args.r#type,
        stream_controller: args.stream_controller,
        unique: true,
    };
    throttled_stream(args.count, args.rate)
        .await
        .for_each_concurrent(None, |sleep_future| async {
            sleep_future.await;
            println!(
                "count: {}, time: {:?}",
                counter.fetch_add(1, Ordering::SeqCst),
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            );
            anchor_request(anchor_request_args.clone()).await.unwrap();
        })
        .await;
    Ok(())
}

async fn throttled_stream(
    count: u64,
    rate: Option<u64>,
) -> Pin<Box<dyn Stream<Item = Sleep> + Send>> {
    if let Some(rate) = rate {
        let start = Instant::now();
        let interval = Duration::from_nanos(1_000_000_000_u64 / rate);
        Box::pin(stream::iter(0..count).map(move |i| {
            let delay_time = start + interval * (i as u32);
            sleep_until(delay_time)
        }))
    } else {
        Box::pin(stream::iter(0_u64..count).map(|_| sleep(Duration::from_secs(0))))
    }
}
