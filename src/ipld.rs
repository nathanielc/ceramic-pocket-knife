use std::{
    io::{self, Cursor, Read},
    str::FromStr,
};

use anyhow::Result;
use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use dag_jose::DagJoseCodec;
use libipld::{
    cbor::DagCborCodec,
    json::DagJsonCodec,
    prelude::{Decode, Encode},
    Ipld,
};
use rand::{thread_rng, Rng};

use crate::cli::{CidInspectArgs, Command};

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

pub fn random_cid() -> Cid {
    let mut data = [0u8; 8];
    thread_rng().fill(&mut data);
    let hash = Code::Sha2_256.digest(&data);
    Cid::new_v1(0x00, hash)
}

pub async fn run(op: Operation) -> Result<()> {
    match op {
        Operation::CidGenerate => {
            let cid = random_cid();
            println!("{}", cid);
        }
        Operation::CidInspect(args) => {
            let cid = Cid::from_str(&args.cid)?;
            println!(
                "CID: {}\nVersion: {:?}\nCodec: 0x{:x}\nHash Code: 0x{:x}\nHash: 0x{}\n",
                cid.into_v1()?,
                cid.version(),
                cid.codec(),
                cid.hash().code(),
                hex::encode(cid.hash().digest())
            );
        }
        Operation::DagJsonToCbor => {
            let mut data = Vec::new();
            io::stdin().read_to_end(&mut data)?;
            let dag_data = Ipld::decode(DagJsonCodec, &mut Cursor::new(data))?;
            let mut out = Vec::new();
            dag_data.encode(DagCborCodec, &mut out)?;
            println!("{}", hex::encode(out));
        }
        Operation::DagJoseToJson => {
            let mut data = Vec::new();
            io::stdin().read_to_end(&mut data)?;
            let dag_data = Ipld::decode(DagJoseCodec, &mut Cursor::new(data))?;
            let mut out = Vec::new();
            dag_data.encode(DagJsonCodec, &mut out)?;
            println!("{}", String::from_utf8(out)?);
        }
    };
    Ok(())
}
