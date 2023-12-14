use std::collections::BTreeMap;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use iroh_car::{CarHeader, CarWriter};
use libipld::{cbor::DagCborCodec, ipld, prelude::Codec, Ipld, IpldCodec};
use multihash::{Code::Sha2_256, MultihashDigest};

use ceramic_core::{Cid, DidDocument, JwkSigner, Jws, StreamId};

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
    // Create a stream and genesis commit
    let (stream_id, genesis_cid, genesis_block) =
        ceramic::create_stream(args.r#type, args.stream_controller, args.deterministic).unwrap();
    // Create a CAR corresponding to the commit
    let (root_cid, car_bytes) = anchor_request_car(
        stream_id,
        genesis_cid,
        genesis_block.clone(),
        // TODO: Pass a tip when we support writing non-genesis commits
        genesis_cid,
        genesis_block,
    )
    .await?;
    // Send the anchor request
    let cas_url = format!("{}/api/v0/requests", args.url);
    let auth_header =
        cas_auth_header(cas_url.clone(), args.controller, args.private_key, root_cid).await?;
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

async fn cas_auth_header(
    url: String,
    controller: String,
    private_key: String,
    digest: Cid,
) -> Result<String> {
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
    let signer = JwkSigner::new(DidDocument::new(controller.as_str()), private_key.as_str())
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

async fn anchor_request_car(
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
