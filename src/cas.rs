use anyhow::Result;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ceramic_core::{Cid, DidDocument, JwkSigner, Jws};

use crate::{
    ceramic,
    cli::{AnchorRequestArgs, Command},
};

pub enum Operation {
    AnchorRequest(AnchorRequestArgs),
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::AnchorRequest(args) => Ok(Operation::AnchorRequest(args)),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation) -> Result<()> {
    match op {
        Operation::AnchorRequest(args) => {
            anchor_request(args).await?;
        }
    };
    Ok(())
}

async fn anchor_request(args: AnchorRequestArgs) -> Result<()> {
    // Create a new stream and its corresponding anchor request CAR bytes
    let (root_cid, car_bytes) =
        ceramic::create_stream_car(args.r#type, args.stream_controller, args.unique).await?;
    // Send the anchor request
    let cas_url = format!("{}/api/v0/requests", args.url);
    let auth_header = cas_auth_header(cas_url.clone(), args.node_controller, root_cid).await?;
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
async fn cas_auth_header(url: String, controller: String, digest: Cid) -> Result<String> {
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
