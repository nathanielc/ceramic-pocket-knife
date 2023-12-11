use std::{io::Cursor, str::FromStr};

use anyhow::Result;
use cid::Cid;
use dag_jose::DagJoseCodec;
use futures::pin_mut;
use libipld::{
    cbor::DagCborCodec,
    json::DagJsonCodec,
    prelude::{Decode, Encode},
    Ipld,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    cli::{CidInspectArgs, Command},
    random_cid,
};

pub enum Operation {
    CidGenerate,
    CidInspect(CidInspectArgs),
    DagJsonToCbor,
    DagJoseToJson,
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::CidGenerate => Ok(Operation::CidGenerate),
            Command::CidInspect(args) => Ok(Operation::CidInspect(args)),
            Command::DagJsonToCbor => Ok(Operation::DagJsonToCbor),
            Command::DagJoseToJson => Ok(Operation::DagJoseToJson),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation, stdin: impl AsyncRead, stdout: impl AsyncWrite) -> Result<()> {
    pin_mut!(stdin, stdout);
    match op {
        Operation::CidGenerate => {
            let cid = random_cid();
            stdout.write_all(cid.to_string().as_bytes()).await?;
        }
        Operation::CidInspect(args) => {
            let cid = Cid::from_str(&args.cid)?;
            stdout
                .write_all(
                    format!(
                        "CID: {}\nVersion: {:?}\nCodec: 0x{:x}\nHash Code: 0x{:x}\nHash: 0x{}\n",
                        cid.into_v1()?,
                        cid.version(),
                        cid.codec(),
                        cid.hash().code(),
                        hex::encode(cid.hash().digest())
                    )
                    .as_bytes(),
                )
                .await?;
        }
        Operation::DagJsonToCbor => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data = Ipld::decode(DagJsonCodec, &mut Cursor::new(data))?;
            let mut out = Vec::new();
            dag_data.encode(DagCborCodec, &mut out)?;
            stdout.write_all(hex::encode(out).as_bytes()).await?;
        }
        Operation::DagJoseToJson => {
            let mut data = Vec::new();
            stdin.read_to_end(&mut data).await?;
            let dag_data = Ipld::decode(DagJoseCodec, &mut Cursor::new(data))?;
            let mut out = Vec::new();
            dag_data.encode(DagJsonCodec, &mut out)?;
            stdout.write_all(&out).await?;
        }
    };
    Ok(())
}
