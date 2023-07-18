use std::str::FromStr;

use ceramic_core::{Cid, EventId, StreamId, StreamIdType};
use cid::multihash::{Code, MultihashDigest};
use multibase::Base;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::cli::{
    Command, CreateStreamIdArgs, GenerateEventIdArgs, GenerateStreamIdArgs, InspectStreamIdArgs,
    Network, StreamType,
};

pub enum Operation {
    GenerateCid,
    CreateStreamId(CreateStreamIdArgs),
    InspectStreamId(InspectStreamIdArgs),
    GenerateStreamId(GenerateStreamIdArgs),
    GenerateEventId(GenerateEventIdArgs),
    GenerateDidKey,
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::CidGenerate => Ok(Operation::GenerateCid),
            Command::StreamIdCreate(args) => Ok(Operation::CreateStreamId(args)),
            Command::StreamIdInspect(args) => Ok(Operation::InspectStreamId(args)),
            Command::StreamIdGenerate(args) => Ok(Operation::GenerateStreamId(args)),
            Command::EventIdGenerate(args) => Ok(Operation::GenerateEventId(args)),
            Command::DidKeyGenerate => Ok(Operation::GenerateDidKey),
            _ => Err(value),
        }
    }
}

pub fn run(op: Operation) -> anyhow::Result<()> {
    match op {
        Operation::GenerateCid => {
            let cid = random_cid();
            println!("{}", cid);
        }
        Operation::CreateStreamId(args) => {
            let stream_id = StreamId {
                r#type: convert_type(args.r#type),
                cid: Cid::from_str(&args.cid)?,
            };
            println!("{}", stream_id.to_string());
        }
        Operation::InspectStreamId(args) => {
            let stream_id = StreamId::from_str(&args.id)?;
            println!("{:?}", stream_id);
        }
        Operation::GenerateStreamId(args) => {
            let stream_id = StreamId {
                r#type: convert_type(args.r#type),
                cid: random_cid(),
            };
            println!("{}", stream_id.to_string());
        }
        Operation::GenerateEventId(args) => {
            let sort_value = args.sort_value.unwrap_or_else(|| {
                StreamId {
                    r#type: StreamIdType::Model,
                    cid: random_cid(),
                }
                .to_string()
            });
            let controller = args.controller.unwrap_or_else(|| {
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect()
            });
            let init_id = args
                .init_id
                .map(|id| StreamId::from_str(&id))
                .transpose()?
                .unwrap_or_else(|| StreamId {
                    r#type: StreamIdType::Model,
                    cid: random_cid(),
                });
            let event_id = EventId::new(
                &convert_network(args.network, Some(thread_rng().gen())),
                &sort_value,
                &controller,
                &init_id.cid,
                thread_rng().gen(),
                &random_cid(),
            );

            println!("{}", event_id.to_hex());
        }
        Operation::GenerateDidKey => {
            let mut buffer = [0; 32];
            thread_rng().fill(&mut buffer);
            println!("did:key:{}", multibase::encode(Base::Base58Btc, &buffer));
        }
    };
    Ok(())
}

fn convert_type(value: StreamType) -> StreamIdType {
    match value {
        StreamType::Model => StreamIdType::Model,
        StreamType::Document => StreamIdType::Document,
    }
}
fn convert_network(value: Network, local_id: Option<u32>) -> ceramic_core::Network {
    match value {
        Network::Mainnet => ceramic_core::Network::Mainnet,
        Network::TestnetClay => ceramic_core::Network::TestnetClay,
        Network::DevUnstable => ceramic_core::Network::DevUnstable,
        Network::Local => ceramic_core::Network::Local(local_id.unwrap()),
        Network::InMemory => ceramic_core::Network::InMemory,
    }
}

fn random_cid() -> Cid {
    let mut data = [0u8; 64];
    thread_rng().fill(&mut data);
    let hash = Code::Sha2_256.digest(&data);
    Cid::new_v1(0x12, hash)
}
