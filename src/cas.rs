use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use chrono::Utc;
use futures::pin_mut;
use futures::stream::{self, Stream, StreamExt};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::time::{sleep, sleep_until, Duration, Instant, Sleep};
use uuid::Uuid;

use iroh_car::{CarHeader, CarWriter};
use libipld::{cbor::DagCborCodec, ipld, prelude::Codec, Ipld, IpldCodec};
use multihash::{Code::Sha2_256, MultihashDigest};

use ceramic_core::{Cid, DidDocument, JwkSigner, Jws, StreamId};

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

pub async fn run(op: Operation, _stdin: impl AsyncRead, stdout: impl AsyncWrite) -> Result<()> {
    pin_mut!(stdout);
    match op {
        Operation::AnchorRequest(args) => {
            stdout
                .write_all(anchor_request(args).await?.as_ref())
                .await?;
        }
        Operation::LoadGen(args) => {
            load_gen(&AtomicU64::new(1), args).await?;
        }
    };
    Ok(())
}

async fn anchor_request(args: CasAnchorRequestArgs) -> Result<String> {
    // Create a stream and genesis commit
    let (stream_id, genesis_cid, genesis_block) =
        ceramic::create_stream(args.r#type, args.stream_controller, args.unique).unwrap();
    // Create stream tip CAR bytes
    let (root_cid, car_bytes) = stream_tip_car(
        stream_id.clone(),
        genesis_cid,
        genesis_block.clone(),
        // TODO: Pass a tip when we support writing non-genesis commits
        genesis_cid,
        genesis_block,
    )
    .await?;
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
    Ok(res.text().await?)
}

/// Create stream tip CAR bytes for use in anchor requests
pub async fn stream_tip_car(
    stream_id: StreamId,
    genesis_cid: Cid,
    genesis_block: Vec<u8>,
    tip_cid: Cid,
    tip_block: Vec<u8>,
) -> Result<(Cid, Vec<u8>)> {
    // Create root block
    let root_block = ipld!({
        "timestamp": Utc::now().to_rfc3339(),
        "streamId": stream_id.to_vec()?,
        "tip": genesis_cid,
    });
    // Encode the root block as CBOR, and compute the CID.
    let ipld_map: BTreeMap<String, Ipld> = libipld::serde::from_ipld(root_block)?;
    let ipld_bytes = DagCborCodec.encode(&ipld_map)?;
    let root_cid = Cid::new_v1(IpldCodec::DagCbor.into(), Sha2_256.digest(&ipld_bytes));
    let car_header = CarHeader::new_v1(vec![root_cid]);
    let mut car_writer = CarWriter::new(car_header, Vec::new());
    // Write root block
    car_writer.write(root_cid, ipld_bytes).await.unwrap();
    // Write genesis CID/block
    car_writer
        .write(genesis_cid, genesis_block.clone())
        .await
        .unwrap();
    // Write tip CID/block
    car_writer.write(tip_cid, tip_block).await.unwrap();
    Ok((root_cid, car_writer.finish().await.unwrap().to_vec()))
}

/// Generate a JWS-signed header for use with CAS Auth
async fn auth_header(url: String, controller: String, digest: Cid) -> Result<String> {
    #[derive(Serialize, Deserialize)]
    struct CasAuthPayload {
        url: String,
        nonce: String,
        digest: String,
    }
    let auth_payload = CasAuthPayload {
        url,
        nonce: Uuid::new_v4().to_string(),
        digest: digest.to_string(),
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
            let cas_res = anchor_request(anchor_request_args.clone()).await.unwrap();
            println!(
                "count: {}, time: {:?}, {}",
                counter.fetch_add(1, Ordering::SeqCst),
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                cas_res
            );
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
