use std::str::FromStr;

use anyhow::Result;
use ceramic_core::{Cid, EventId, Interest, StreamId, StreamIdType};
use libp2p::futures::pin_mut;
use libp2p_identity::PeerId;
use multibase::Base;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use recon::{AssociativeHash, Key, Sha256a};
use sqlx::{Connection, QueryBuilder, Sqlite, SqliteConnection};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::{
    cli::{
        Command, EventIdDecodeArgs, EventIdGenerateArgs, InterestDecodeArgs, Network,
        SqlDbGenerateArgs, StreamIdCreateArgs, StreamIdGenerateArgs, StreamIdInspectArgs,
        StreamType,
    },
    ipld::random_cid,
};

pub enum Operation {
    StreamIdCreate(StreamIdCreateArgs),
    StreamIdInspect(StreamIdInspectArgs),
    StreamIdGenerate(StreamIdGenerateArgs),
    EventIdGenerate(EventIdGenerateArgs),
    EventIdDecode(EventIdDecodeArgs),
    InterestDecode(InterestDecodeArgs),
    DidKeyGenerate,
    PeerIdGenerate,
    SqlDbGenerate(SqlDbGenerateArgs),
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::StreamIdCreate(args) => Ok(Operation::StreamIdCreate(args)),
            Command::StreamIdInspect(args) => Ok(Operation::StreamIdInspect(args)),
            Command::StreamIdGenerate(args) => Ok(Operation::StreamIdGenerate(args)),
            Command::EventIdGenerate(args) => Ok(Operation::EventIdGenerate(args)),
            Command::EventIdDecode(args) => Ok(Operation::EventIdDecode(args)),
            Command::InterestDecode(args) => Ok(Operation::InterestDecode(args)),
            Command::DidKeyGenerate => Ok(Operation::DidKeyGenerate),
            Command::PeerIdGenerate => Ok(Operation::PeerIdGenerate),
            Command::SqlDbGenerate(args) => Ok(Operation::SqlDbGenerate(args)),
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
            stdout.write_all(stream_id.to_string().as_bytes()).await?;
        }
        Operation::StreamIdInspect(args) => {
            let stream_id = StreamId::from_str(&args.id)?;
            stdout
                .write_all(format!("{:?}", stream_id).as_bytes())
                .await?;
        }
        Operation::StreamIdGenerate(args) => {
            let stream_id = StreamId {
                r#type: convert_type(args.r#type),
                cid: random_cid(),
            };
            stdout.write_all(stream_id.to_string().as_bytes()).await?;
        }
        Operation::EventIdGenerate(args) => {
            let network = &convert_network(
                args.network,
                Some(args.local_network_id.unwrap_or_else(|| thread_rng().gen())),
            );
            let event_id = random_event_id(
                &network,
                args.sort_key,
                args.sort_value,
                args.controller,
                args.init_id,
            )?;
            stdout.write_all(event_id.to_hex().as_bytes()).await?;
        }
        Operation::EventIdDecode(args) => {
            let bytes = hex::decode(args.event_id)?;
            let event_id = EventId::from(bytes);
            stdout
                .write_all(format!("{:#?}", event_id).as_bytes())
                .await?;
        }
        Operation::InterestDecode(args) => {
            let bytes = hex::decode(args.interest)?;
            let interest = Interest::from(bytes);
            stdout
                .write_all(format!("{:#?}", interest).as_bytes())
                .await?;
        }
        Operation::DidKeyGenerate => {
            let mut buffer = [0; 32];
            thread_rng().fill(&mut buffer);
            stdout
                .write_all(
                    format!("did:key:{}", multibase::encode(Base::Base58Btc, &buffer)).as_bytes(),
                )
                .await?;
        }
        Operation::PeerIdGenerate => {
            let peer_id = PeerId::random();
            stdout.write_all(peer_id.to_string().as_bytes()).await?;
        }
        Operation::SqlDbGenerate(args) => {
            let network = &convert_network(args.network, Some(thread_rng().gen()));

            let mut conn =
                SqliteConnection::connect(&format!("sqlite:{}?mode=rwc", args.path.display()))
                    .await?;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS recon (
            sort_key TEXT, -- the field in the event header to sort by e.g. model
            key BLOB, -- network_id sort_value controller StreamID height event_cid
            ahash_0 INTEGER, -- the ahash is decomposed as [u32; 8]
            ahash_1 INTEGER,
            ahash_2 INTEGER,
            ahash_3 INTEGER,
            ahash_4 INTEGER,
            ahash_5 INTEGER,
            ahash_6 INTEGER,
            ahash_7 INTEGER,
            CID TEXT,
            block_retrieved BOOL, -- indicates if we still want the block
            PRIMARY KEY(sort_key, key)
        )",
            )
            .execute(&mut conn)
            .await?;

            let mut batch = Vec::with_capacity(1000);
            for _ in 0..args.count {
                let event_id = random_event_id(
                    &network,
                    Some(args.sort_key.clone()),
                    args.sort_value.clone(),
                    args.controller.clone(),
                    args.init_id.clone(),
                )?;
                batch.push(event_id);
                if batch.len() == batch.capacity() {
                    insert_batch(&mut conn, &args.sort_key, &batch).await?;
                    batch.clear();
                }
            }
            insert_batch(&mut conn, &args.sort_key, &batch).await?;
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

async fn insert_batch(
    conn: &mut SqliteConnection,
    sort_key: &str,
    batch: &[EventId],
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
                        // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
                        // spaces as that might interfere with identifiers or quoted strings where exact
                        // values may matter.
                        "INSERT INTO recon ( sort_key, key, ahash_0, ahash_1, ahash_2, ahash_3, ahash_4, ahash_5, ahash_6, ahash_7, block_retrieved) ",
                    );
    query_builder.push_values(batch.into_iter(), |mut b, event| {
        let hash = Sha256a::digest(event);
        b.push_bind(sort_key)
            .push_bind(event.as_bytes())
            .push_bind(hash.as_u32s()[0])
            .push_bind(hash.as_u32s()[1])
            .push_bind(hash.as_u32s()[2])
            .push_bind(hash.as_u32s()[3])
            .push_bind(hash.as_u32s()[4])
            .push_bind(hash.as_u32s()[5])
            .push_bind(hash.as_u32s()[6])
            .push_bind(hash.as_u32s()[7])
            .push_bind(false);
    });
    let query = query_builder.build();
    query.execute(conn).await?;
    Ok(())
}
