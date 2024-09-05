use std::str::FromStr;

use cid::Cid;
use clap::{Args, Parser, Subcommand, ValueEnum};

/// Convert to and from multibase encodings
#[derive(Parser, Debug)]
#[command(name = "cpk", author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Generate shell completion scripts to stdout, i.e. for bash run: source <(cpk completion bash)
    Completion(CompletionArgs),
    // ---------------- Multibase Tools ----------------------------//
    /// Decode encoded input from stdin without the multibase prefix.
    BaseGuess,
    /// Decode multi-base encoded input from stdin.
    BaseDecode,
    /// convert stdin to base2            0 binary (01010101)
    Base2,
    /// convert stdin to base8            7 octal
    Base8,
    /// convert stdin to base10           9 decimal
    Base10,
    /// convert stdin to base16           f hexadecimal
    Base16,
    /// convert stdin to base16-upper     F hexadecimal
    Base16Upper,
    /// convert stdin to base32-hex       v rfc4648 no padding - highest char
    Base32Hex,
    /// convert stdin to base32-hex-upper V rfc4648 no padding - highest char
    Base32HexUpper,
    /// convert stdin to base32           b rfc4648 no padding
    Base32,
    /// convert stdin to base32-upper     B rfc4648 no padding
    Base32Upper,
    /// convert stdin to base32-z         h z-base-32 (used by Tahoe-LAFS)
    Base32Z,
    /// convert stdin to base36           k lowercase alphanumeric no padding
    Base36,
    /// convert stdin to base36-upper     K uppercase alphanumeric no padding
    Base36Upper,
    /// convert stdin to base58-flickr    Z base58 flicker
    Base58Flickr,
    /// convert stdin to base58-btc       z base58 bitcoin
    Base58Btc,
    /// convert stdin to base64           m rfc4648 no padding
    Base64,
    /// convert stdin to base64-url       u rfc4648 no padding
    Base64Url,

    // ---------------- Multihash Tools ----------------------------//
    MultihashInspect,

    // ---------------- Ceramic Tools ----------------------------//
    /// Create a stream ID
    StreamIdCreate(StreamIdCreateArgs),
    /// Inspect a stream ID
    StreamIdInspect(StreamIdInspectArgs),
    /// Generate a random stream ID
    StreamIdGenerate(StreamIdGenerateArgs),
    /// Construct a stream ID from raw bytes
    StreamIdFromBytes,
    /// Generate a random event ID
    EventIdGenerate(EventIdGenerateArgs),
    /// Inspect a multibase encoded event ID
    EventIdInspect(EventIdInspectArgs),
    /// Inspect an event car file
    EventInspect,
    /// Inspect a multibase encoded interest
    InterestInspect(InterestInspectArgs),
    /// Generate a random did:key method
    DidKeyGenerate,
    /// Generate a random peer ID
    PeerIdGenerate,

    // ---------------- IPLD Tools ----------------------------//
    /// Generate a random stream ID
    CidGenerate,
    /// Inspect a CID
    CidInspect(CidInspectArgs),
    /// Construct a CID from CID bytes
    CidFromBytes,
    /// Hash bytes to compute a CID
    CidFromData(CidFromDataArgs),
    /// Convert DAG-JSON data to DAG-CBOR
    DagJsonToCbor,
    /// Convert DAG-CBOR data to DAG-JSON
    DagCborToJson,
    /// Convert DAG-JOSE data to DAG-JSON
    DagJoseToJson,
    /// Inspect DAG-CBOR encoded data
    DagCborInspect,
    /// Index into DAG-CBOR encoded data
    DagCborIndex(DagCborIndexArgs),
    /// List contents of a CAR archive
    CarInspect(CarInspectArgs),
    /// Extract a single root CID from a CAR archive
    CarExtract(CarExtractArgs),
    /// Construct a CAR file bytes from a list of blocks
    CarFromBlocks(CarFromBlocksArgs),

    // ---------------- Libp2p Tools ----------------------------//
    P2pPing(PingArgs),
    P2pIdentify(IdentifyArgs),
}

#[derive(Args, Debug, Clone)]
pub struct CompletionArgs {
    /// Shell type.
    #[arg(value_enum)]
    pub shell: clap_complete_command::Shell,
}

#[derive(Args, Debug, Clone)]
pub struct StreamIdCreateArgs {
    /// Stream type.
    #[arg(long, value_enum)]
    pub r#type: StreamType,
    /// Init CID of the stream
    #[arg(long)]
    pub cid: String,
}

#[derive(Args, Debug, Clone)]
pub struct StreamIdInspectArgs {
    /// Stream ID
    #[arg()]
    pub id: String,
}

#[derive(Args, Debug, Clone)]
pub struct StreamIdGenerateArgs {
    /// Stream type.
    #[arg(long, value_enum)]
    pub r#type: StreamType,
}

#[derive(Args, Debug, Clone)]
pub struct EventIdGenerateArgs {
    /// Network
    #[arg(long, default_value = "testnet-clay", value_enum)]
    pub network: Network,
    /// Local Network ID, only used when network is local. If not set a random ID is used.
    #[arg(long)]
    pub local_network_id: Option<u32>,
    /// Sort Key, if not set generates random value.
    #[arg(long)]
    pub sort_key: Option<String>,
    /// Multibase encoded sort value, if not set generates random value.
    #[arg(long)]
    pub sort_value: Option<String>,
    /// Controller, if not set generates random value.
    #[arg(long)]
    pub controller: Option<String>,
    /// Stream ID of init event, if not set generates random value.
    #[arg(long)]
    pub init_id: Option<String>,
}
#[derive(Args, Debug, Clone)]
pub struct EventIdInspectArgs {
    /// Multibase encoded Event ID to decode
    #[arg()]
    pub event_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct InterestInspectArgs {
    /// Hex encoded Interest to decode
    #[arg()]
    pub interest: String,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Network {
    Mainnet,
    TestnetClay,
    DevUnstable,
    Local,
    InMemory,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum StreamType {
    Model,
    Document,
}

#[derive(Args, Debug, Clone)]
pub struct CidInspectArgs {
    /// CID, if `-` will read CID from STDIN as a string.
    #[arg()]
    pub cid: String,
}

#[derive(Args, Debug, Clone)]
pub struct CidFromDataArgs {
    /// Codec
    #[arg()]
    pub codec: u64,
}

#[derive(Args, Debug, Clone)]
pub struct CarInspectArgs {
    /// When true, only metadata about the car file is decoded
    #[arg(long, default_value_t = false)]
    pub metadata_only: bool,
}

#[derive(Args, Debug, Clone)]
pub struct CarExtractArgs {
    /// CID
    #[arg()]
    pub cid: String,
}

#[derive(Args, Debug, Clone)]
pub struct CarFromBlocksArgs {
    /// List of files to add to the car file in order.
    /// Format the arg with `@cid:path/to/file` for blocks that are part of the roots and
    /// format with `cid:path/to/file` for blocks that are NOT part of the roots.
    #[arg()]
    pub blocks: Vec<CarBlockValue>,
}

#[derive(Debug, Clone)]
pub struct CarBlockValue {
    pub root: bool,
    pub cid: Cid,
    pub path: String,
}

impl FromStr for CarBlockValue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const ROOT_PATTERN: char = '@';
        let root = s.starts_with(ROOT_PATTERN);
        let (cid, path) = s
            .trim_start_matches(ROOT_PATTERN)
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("car block must be in the form cid:path/to/block"))?;
        Ok(Self {
            root,
            cid: Cid::from_str(cid)?,
            path: path.to_string(),
        })
    }
}

#[derive(Args, Debug, Clone)]
pub struct DagCborIndexArgs {
    /// Index path into the IPLD value
    #[arg()]
    pub index: String,
}

#[derive(Args, Debug, Clone)]
pub struct PingArgs {
    /// Multiaddr for Peer
    #[arg()]
    pub peer_addr: String,

    /// Number of pings to send before exiting.
    #[arg(short, long, default_value_t = usize::MAX)]
    pub count: usize,

    /// Interval in seconds between pings
    #[arg(short, long, default_value_t = 1)]
    pub interval: u32,

    /// Timeout in seconds to wait for a pong
    #[arg(short, long, default_value_t = 20)]
    pub timeout: u32,
}
#[derive(Args, Debug, Clone)]
pub struct IdentifyArgs {
    /// Multiaddr for Peer
    #[arg()]
    pub peer_addr: String,
}
