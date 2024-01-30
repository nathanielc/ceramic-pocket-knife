use std::{io::Cursor, str::FromStr};

use anyhow::Result;
use cid::Cid;
use dag_jose::DagJoseCodec;
use futures::pin_mut;
use iroh_car::CarReader;
use libipld::{
    cbor::DagCborCodec,
    json::DagJsonCodec,
    prelude::{Decode, Encode},
    Ipld,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    cli::{CarExtractArgs, CarInspectArgs, CidInspectArgs, Command},
    random_cid,
};

pub enum Operation {
    CidGenerate,
    CidInspect(CidInspectArgs),
    DagJsonToCbor,
    DagCborToJson,
    DagJoseToJson,
    CarInspect(CarInspectArgs),
    CarExtract(CarExtractArgs),
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::CidGenerate => Ok(Operation::CidGenerate),
            Command::CidInspect(args) => Ok(Operation::CidInspect(args)),
            Command::DagJsonToCbor => Ok(Operation::DagJsonToCbor),
            Command::DagCborToJson => Ok(Operation::DagCborToJson),
            Command::DagJoseToJson => Ok(Operation::DagJoseToJson),
            Command::CarInspect(args) => Ok(Operation::CarInspect(args)),
            Command::CarExtract(args) => Ok(Operation::CarExtract(args)),
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
            stdout.write_all(cid.to_string().as_bytes()).await?;
        }
        Operation::CidInspect(args) => {
            let cid = Cid::from_str(&args.cid)?;
            stdout.write_all(fmt_cid(&cid)?.as_bytes()).await?;
        }
        Operation::DagJsonToCbor => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data = Ipld::decode(DagJsonCodec, &mut Cursor::new(data))?;
            let mut out = Vec::new();
            dag_data.encode(DagCborCodec, &mut out)?;
            stdout.write_all(hex::encode(out).as_bytes()).await?;
        }
        Operation::DagCborToJson => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data = Ipld::decode(DagCborCodec, &mut Cursor::new(data))?;
            let mut out = Vec::new();
            dag_data.encode(DagJsonCodec, &mut out)?;
            stdout.write_all(&out).await?;
            stdout.write_all(b"\n").await?;
        }
        Operation::DagJoseToJson => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data = Ipld::decode(DagJoseCodec, &mut Cursor::new(data))?;
            let mut out = Vec::new();
            dag_data.encode(DagJsonCodec, &mut out)?;
            stdout.write_all(&out).await?;
            stdout.write_all(b"\n").await?;
        }
        Operation::CarInspect(args) => {
            let mut reader = CarReader::new(stdin).await?;
            while let Some((cid, data)) = reader.next_block().await? {
                stdout.write_all(fmt_cid(&cid)?.as_bytes()).await?;
                stdout
                    .write_all(format!("Length: {}\n", data.len()).as_bytes())
                    .await?;
                if !args.metadata_only {
                    let dag_data = match cid.codec() {
                        0x71 => Some(Ipld::decode(DagCborCodec, &mut Cursor::new(data))?),
                        0x129 => Some(Ipld::decode(DagJsonCodec, &mut Cursor::new(data))?),
                        0x85 => Some(Ipld::decode(DagJoseCodec, &mut Cursor::new(data))?),
                        _ => None,
                    };
                    if let Some(dag_data) = dag_data {
                        let mut out = Vec::new();
                        dag_data.encode(DagJsonCodec, &mut out)?;
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
    };
    Ok(())
}

fn fmt_cid(cid: &Cid) -> Result<String> {
    Ok(format!(
        "CID: {}\nVersion: {:?}\nCodec: 0x{:x}\nHash Code: 0x{:x}\nHash: 0x{}\n",
        cid.into_v1()?,
        cid.version(),
        cid.codec(),
        cid.hash().code(),
        hex::encode(cid.hash().digest())
    ))
}
