use std::{io::Cursor, str::FromStr};

use anyhow::{anyhow, bail, Result};
use cid::Cid;
use dag_jose::DagJoseCodec;
use futures::pin_mut;
use ipld_core::{codec::Codec, ipld::Ipld};
use iroh_car::{CarHeader, CarReader, CarWriter};
use multihash_codetable::{Code, MultihashDigest};
use serde_ipld_dagcbor::codec::DagCborCodec;
use serde_ipld_dagjson::codec::DagJsonCodec;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    cli::{
        CarExtractArgs, CarFromBlocksArgs, CarInspectArgs, CidFromDataArgs, CidInspectArgs,
        Command, DagCborIndexArgs,
    },
    random_cid,
};

pub enum Operation {
    CidGenerate,
    CidInspect(CidInspectArgs),
    CidFromBytes,
    CidFromData(CidFromDataArgs),
    DagJsonToCbor,
    DagCborToJson,
    DagJoseToJson,
    DagCborInspect,
    DagCborIndex(DagCborIndexArgs),
    CarInspect(CarInspectArgs),
    CarExtract(CarExtractArgs),
    CarFromBlocks(CarFromBlocksArgs),
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::CidGenerate => Ok(Operation::CidGenerate),
            Command::CidInspect(args) => Ok(Operation::CidInspect(args)),
            Command::CidFromBytes => Ok(Operation::CidFromBytes),
            Command::CidFromData(args) => Ok(Operation::CidFromData(args)),
            Command::DagJsonToCbor => Ok(Operation::DagJsonToCbor),
            Command::DagCborToJson => Ok(Operation::DagCborToJson),
            Command::DagJoseToJson => Ok(Operation::DagJoseToJson),
            Command::DagCborInspect => Ok(Operation::DagCborInspect),
            Command::DagCborIndex(args) => Ok(Operation::DagCborIndex(args)),
            Command::CarInspect(args) => Ok(Operation::CarInspect(args)),
            Command::CarExtract(args) => Ok(Operation::CarExtract(args)),
            Command::CarFromBlocks(args) => Ok(Operation::CarFromBlocks(args)),
            _ => Err(value),
        }
    }
}

pub async fn run(
    op: Operation,
    stdin: impl AsyncRead + Send,
    stdout: impl AsyncWrite,
) -> Result<()> {
    pin_mut!(stdin, stdout);
    match op {
        Operation::CidGenerate => {
            let cid = random_cid();
            stdout.write_all(format!("{cid}\n").as_bytes()).await?;
        }
        Operation::CidInspect(args) => {
            let cid = if args.cid == "-" {
                let mut data = Vec::new();
                stdin.read_to_end(&mut data).await?;
                let cid_str = std::str::from_utf8(&data)?.trim();
                Cid::from_str(cid_str)?
            } else {
                Cid::from_str(&args.cid)?
            };
            stdout.write_all(fmt_cid(&cid)?.as_bytes()).await?;
        }
        Operation::CidFromBytes => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let cid = Cid::read_bytes(Cursor::new(data))?;
            stdout.write_all(format!("{cid}\n").as_bytes()).await?;
        }
        Operation::CidFromData(args) => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let hash = Code::Sha2_256.digest(&data);
            let cid = Cid::new_v1(args.codec, hash);
            stdout.write_all(format!("{cid}\n").as_bytes()).await?;
        }
        Operation::DagJsonToCbor => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data: Ipld = serde_ipld_dagjson::from_slice(&data)?;
            let out = serde_ipld_dagcbor::to_vec(&dag_data)?;
            stdout
                .write_all(format!("{}\n", hex::encode(out)).as_bytes())
                .await?;
        }
        Operation::DagCborToJson => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data: Ipld = serde_ipld_dagcbor::from_slice(&data)?;
            let out = serde_ipld_dagjson::to_vec(&dag_data)?;
            stdout.write_all(&out).await?;
            stdout.write_all(b"\n").await?;
        }
        Operation::DagJoseToJson => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data: Ipld = DagJoseCodec::decode_from_slice(&data)?;
            let out = serde_ipld_dagjson::to_vec(&dag_data)?;
            stdout.write_all(&out).await?;
            stdout.write_all(b"\n").await?;
        }
        Operation::DagCborInspect => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data: Ipld = serde_ipld_dagcbor::from_slice(&data)?;
            stdout
                .write_all(format!("{dag_data:#?}\n").as_bytes())
                .await?;
        }
        Operation::DagCborIndex(args) => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data: Ipld = serde_ipld_dagcbor::from_slice(&data)?;
            let mut idx_data = dag_data;
            for index in args.index.split('/') {
                idx_data = idx_data
                    .take(index)?
                    .ok_or_else(|| anyhow!("no IPLD data exists at index"))?;
            }
            match idx_data {
                // Write nothing for Null values
                Ipld::Null => {}
                Ipld::Bytes(bytes) => stdout.write_all(&bytes).await?,
                composite @ Ipld::List(_) | composite @ Ipld::Map(_) => {
                    stdout
                        .write_all(&serde_ipld_dagcbor::to_vec(&composite)?)
                        .await?;
                }
                Ipld::Bool(b) => stdout.write_all(format!("{b}\n").as_bytes()).await?,
                Ipld::Integer(i) => stdout.write_all(format!("{i}\n").as_bytes()).await?,
                Ipld::Float(f) => stdout.write_all(format!("{f}\n").as_bytes()).await?,
                Ipld::String(s) => stdout.write_all(format!("{s}\n").as_bytes()).await?,
                Ipld::Link(cid) => stdout.write_all(&cid.to_bytes()).await?,
            };
        }
        Operation::CarInspect(args) => {
            let mut reader = CarReader::new(stdin).await?;
            let roots: Vec<Cid> = reader.header().roots().to_vec();
            while let Some((cid, data)) = reader.next_block().await? {
                stdout.write_all(fmt_cid(&cid)?.as_bytes()).await?;
                stdout
                    .write_all(
                        format!("Root: {}\nLength: {}\n", roots.contains(&cid), data.len())
                            .as_bytes(),
                    )
                    .await?;
                if !args.metadata_only {
                    let dag_data: Option<Ipld> = match cid.codec() {
                        <DagCborCodec as Codec<Ipld>>::CODE => {
                            Some(serde_ipld_dagcbor::from_slice(&data)?)
                        }
                        <DagJsonCodec as Codec<Ipld>>::CODE => {
                            Some(serde_ipld_dagjson::from_slice(&data)?)
                        }
                        <DagJoseCodec as Codec<Ipld>>::CODE => {
                            Some(DagJoseCodec::decode_from_slice(&data)?)
                        }
                        _ => None,
                    };
                    if let Some(dag_data) = dag_data {
                        let out = serde_ipld_dagjson::to_vec(&dag_data)?;
                        stdout.write_all(&out).await?;
                        stdout.write_all(b"\n").await?;
                    }
                }
            }
        }
        Operation::CarExtract(args) => {
            let find_cid = Cid::from_str(&args.cid)?;
            let mut reader = CarReader::new(stdin).await?;
            while let Some((cid, data)) = reader.next_block().await? {
                if cid == find_cid {
                    stdout.write_all(&data).await?;
                }
            }
        }
        Operation::CarFromBlocks(args) => {
            let mut car = Vec::new();
            let roots: Vec<Cid> = args
                .blocks
                .iter()
                .flat_map(|block| block.root.then_some(block.cid))
                .collect();

            let mut writer = CarWriter::new(CarHeader::V1(roots.into()), &mut car);
            for block in args.blocks {
                eprintln!("adding block to car file: {block:?}");
                let data = tokio::fs::read(block.path).await?;

                let hash = MultihashDigest::digest(
                    &multihash_codetable::Code::try_from(block.cid.hash().code())?,
                    &data,
                );
                if hash != *block.cid.hash() {
                    bail!("Cid {} hash does not match hash of raw data", block.cid)
                }

                writer.write(block.cid, data).await?;
            }
            writer.finish().await?;

            stdout.write_all(&car).await?;
        }
    };
    Ok(())
}

fn fmt_cid(cid: &Cid) -> Result<String> {
    let (v0_str, v0_bytes) = Cid::new_v0(*cid.hash())
        .map(|v0| (v0.to_string(), v0.to_bytes()))
        .unwrap_or_default();
    Ok(format!(
        "CID V1: {}\nCID V0: {}\nBase32: {}\nVersion: {:?}\nCodec: 0x{:x}\nHash Code: 0x{:x}\nHash: 0x{}\n",
        cid.into_v1()?,
        &v0_str,
        multibase::encode(multibase::Base::Base32Upper, &v0_bytes),
        cid.version(),
        cid.codec(),
        cid.hash().code(),
        hex::encode(cid.hash().digest())
    ))
}
