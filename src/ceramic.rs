use std::{
    io::{stdout, Write},
    str::FromStr,
};

use anyhow::Result;
use ceramic_core::{Cid, EventId, Interest, StreamId, StreamIdType};
use cid::multihash::{Code, MultihashDigest};
use libp2p_identity::PeerId;
use multibase::Base;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use recon::{AssociativeHash, Key, Sha256a};
use sqlx::{Connection, QueryBuilder, Sqlite, SqliteConnection};

use crate::cli::{
    Command, CreateStreamIdArgs, DecodeEventIdArgs, DecodeInterestArgs, GenerateEventIdArgs,
    GenerateSqlDbArgs, GenerateStreamIdArgs, InspectStreamIdArgs, Network, StreamType,
};

pub enum Operation {
    GenerateCid,
    CreateStreamId(CreateStreamIdArgs),
    InspectStreamId(InspectStreamIdArgs),
    GenerateStreamId(GenerateStreamIdArgs),
    GenerateEventId(GenerateEventIdArgs),
    DecodeEventId(DecodeEventIdArgs),
    DecodeInterest(DecodeInterestArgs),
    GenerateDidKey,
    GeneratePeerId,
    GenerateSqlDb(GenerateSqlDbArgs),
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
            Command::EventIdDecode(args) => Ok(Operation::DecodeEventId(args)),
            Command::InterestDecode(args) => Ok(Operation::DecodeInterest(args)),
            Command::DidKeyGenerate => Ok(Operation::GenerateDidKey),
            Command::PeerIdGenerate => Ok(Operation::GeneratePeerId),
            Command::SqlDbGenerate(args) => Ok(Operation::GenerateSqlDb(args)),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation) -> Result<()> {
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
            let network = &convert_network(args.network, Some(thread_rng().gen()));
            let event_id = random_event_id(
                &network,
                args.sort_key,
                args.sort_value,
                args.controller,
                args.init_id,
            )?;
            println!("{}", event_id.to_hex());
        }
        Operation::DecodeEventId(args) => {
            let bytes = hex::decode(args.event_id)?;
            let event_id = EventId::from(bytes);
            println!("{:#?}", event_id);
        }
        Operation::DecodeInterest(args) => {
            let bytes = hex::decode(args.interest)?;
            let interest = Interest::from(bytes);
            println!("{:#?}", interest);
        }
        Operation::GenerateDidKey => {
            let mut buffer = [0; 32];
            thread_rng().fill(&mut buffer);
            println!("did:key:{}", multibase::encode(Base::Base58Btc, &buffer));
        }
        Operation::GeneratePeerId => {
            let peer_id = PeerId::random();
            println!("{}", peer_id.to_string());
        }
        Operation::GenerateSqlDb(args) => {
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

            print!("writing {} events", args.count);
            stdout().flush()?;
            let mut batch = Vec::with_capacity(1000);
            for i in 0..args.count {
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
                if i % 100000 == 0 {
                    print!(".");
                    stdout().flush()?;
                }
            }
            insert_batch(&mut conn, &args.sort_key, &batch).await?;
            println!("done");
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

fn random_cid() -> Cid {
    let mut data = [0u8; 8];
    thread_rng().fill(&mut data);
    let hash = Code::Sha2_256.digest(&data);
    Cid::new_v1(0x00, hash)
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
