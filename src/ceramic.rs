use std::str::FromStr;

use anyhow::Result;
use ceramic_core::{Cid, EventId, Interest, StreamId, StreamIdType};
use futures::pin_mut;
use libp2p_identity::PeerId;
use multibase::Base;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use recon::Key;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::{
    cli::{
        Command, EventIdGenerateArgs, EventIdInspectArgs, InterestInspectArgs, Network,
        StreamIdCreateArgs, StreamIdGenerateArgs, StreamIdInspectArgs, StreamType,
    },
    random_cid,
};

pub enum Operation {
    StreamIdCreate(StreamIdCreateArgs),
    StreamIdInspect(StreamIdInspectArgs),
    StreamIdGenerate(StreamIdGenerateArgs),
    EventIdGenerate(EventIdGenerateArgs),
    EventIdInspect(EventIdInspectArgs),
    InterestInspect(InterestInspectArgs),
    DidKeyGenerate,
    PeerIdGenerate,
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::StreamIdCreate(args) => Ok(Operation::StreamIdCreate(args)),
            Command::StreamIdInspect(args) => Ok(Operation::StreamIdInspect(args)),
            Command::StreamIdGenerate(args) => Ok(Operation::StreamIdGenerate(args)),
            Command::EventIdGenerate(args) => Ok(Operation::EventIdGenerate(args)),
            Command::EventIdInspect(args) => Ok(Operation::EventIdInspect(args)),
            Command::InterestInspect(args) => Ok(Operation::InterestInspect(args)),
            Command::DidKeyGenerate => Ok(Operation::DidKeyGenerate),
            Command::PeerIdGenerate => Ok(Operation::PeerIdGenerate),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation, _stdin: impl AsyncRead, stdout: impl AsyncWrite) -> Result<()> {
    pin_mut!(stdout);
    match op {
        Operation::StreamIdCreate(args) => {
            let stream_id = StreamId {
                r#type: convert_type(args.r#type),
                cid: Cid::from_str(&args.cid)?,
            };
            stdout
                .write_all(format!("{:?}\n", stream_id).as_bytes())
                .await?;
        }
        Operation::StreamIdInspect(args) => {
            let stream_id = StreamId::from_str(&args.id)?;
            stdout
                .write_all(format!("{:?}\n", stream_id).as_bytes())
                .await?;
        }
        Operation::StreamIdGenerate(args) => {
            let stream_id = StreamId {
                r#type: convert_type(args.r#type),
                cid: random_cid(),
            };
            stdout
                .write_all(format!("{:?}\n", stream_id).as_bytes())
                .await?;
        }
        Operation::EventIdGenerate(args) => {
            let network = &convert_network(
                args.network,
                Some(args.local_network_id.unwrap_or_else(|| thread_rng().gen())),
            );
            let event_id = random_event_id(
                network,
                args.sort_key,
                args.sort_value,
                args.controller,
                args.init_id,
            )?;
            stdout
                .write_all(format!("{}\n", event_id.to_hex()).as_bytes())
                .await?;
        }
        Operation::EventIdInspect(args) => {
            let (_base, bytes) = multibase::decode(args.event_id)?;
            let event_id = EventId::try_from(bytes)?;
            stdout
                .write_all(format!("{:#?}\n", event_id).as_bytes())
                .await?;
        }
        Operation::InterestInspect(args) => {
            let (_base, bytes) = multibase::decode(args.interest)?;
            let interest = Interest::try_from(bytes)?;
            stdout
                .write_all(format!("{:#?}\n", interest).as_bytes())
                .await?;
        }
        Operation::DidKeyGenerate => {
            let mut buffer = [0; 32];
            thread_rng().fill(&mut buffer);
            stdout
                .write_all(
                    format!("did:key:{}\n", multibase::encode(Base::Base58Btc, buffer)).as_bytes(),
                )
                .await?;
        }
        Operation::PeerIdGenerate => {
            let peer_id = PeerId::random();
            stdout.write_all(format!("{peer_id}\n").as_bytes()).await?;
        }
    };
    Ok(())
}

fn convert_type(value: StreamType) -> StreamIdType {
    match value {
        StreamType::Model => StreamIdType::Model,
        StreamType::Document => StreamIdType::ModelInstanceDocument,
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

fn random_event_id(
    network: &ceramic_core::Network,
    sort_key: Option<String>,
    sort_value: Option<String>,
    controller: Option<String>,
    init_id: Option<String>,
) -> Result<EventId> {
    let sort_key = sort_key.unwrap_or_else(|| {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect()
    });
    let sort_value = sort_value.unwrap_or_else(|| {
        StreamId {
            r#type: StreamIdType::Model,
            cid: random_cid(),
        }
        .to_string()
    });
    let controller = controller.unwrap_or_else(|| {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    });
    let init_id = init_id
        .map(|id| StreamId::from_str(&id))
        .transpose()?
        .unwrap_or_else(|| StreamId {
            r#type: StreamIdType::Model,
            cid: random_cid(),
        });
    Ok(EventId::new(
        network,
        &sort_key,
        &sort_value,
        &controller,
        &init_id.cid,
        thread_rng().gen(),
        &random_cid(),
    ))
}
