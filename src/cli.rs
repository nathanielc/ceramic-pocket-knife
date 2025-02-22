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
    #[cfg(feature = "multibase")]
    BaseGuess,
    /// Decode multi-base encoded input from stdin.
    #[cfg(feature = "multibase")]
    BaseDecode,
    /// convert stdin to base2            0 binary (01010101)
    #[cfg(feature = "multibase")]
    Base2,
    /// convert stdin to base8            7 octal
    #[cfg(feature = "multibase")]
    Base8,
    /// convert stdin to base10           9 decimal
    #[cfg(feature = "multibase")]
    Base10,
    /// convert stdin to base16           f hexadecimal
    #[cfg(feature = "multibase")]
    Base16,
    /// convert stdin to base16-upper     F hexadecimal
    #[cfg(feature = "multibase")]
    Base16Upper,
    /// convert stdin to base32-hex       v rfc4648 no padding - highest char
    #[cfg(feature = "multibase")]
    Base32Hex,
    /// convert stdin to base32-hex-upper V rfc4648 no padding - highest char
    #[cfg(feature = "multibase")]
    Base32HexUpper,
    /// convert stdin to base32           b rfc4648 no padding
    #[cfg(feature = "multibase")]
    Base32,
    /// convert stdin to base32-upper     B rfc4648 no padding
    #[cfg(feature = "multibase")]
    Base32Upper,
    /// convert stdin to base32-z         h z-base-32 (used by Tahoe-LAFS)
    #[cfg(feature = "multibase")]
    Base32Z,
    /// convert stdin to base36           k lowercase alphanumeric no padding
    #[cfg(feature = "multibase")]
    Base36,
    /// convert stdin to base36-upper     K uppercase alphanumeric no padding
    #[cfg(feature = "multibase")]
    Base36Upper,
    /// convert stdin to base58-flickr    Z base58 flicker
    #[cfg(feature = "multibase")]
    Base58Flickr,
    /// convert stdin to base58-btc       z base58 bitcoin
    #[cfg(feature = "multibase")]
    Base58Btc,
    /// convert stdin to base64           m rfc4648 no padding
    #[cfg(feature = "multibase")]
    Base64,
    /// convert stdin to base64-url       u rfc4648 no padding
    #[cfg(feature = "multibase")]
    Base64Url,

    // ---------------- Multihash Tools ----------------------------//
    /// Inspect a multihash
    #[cfg(feature = "multihash")]
    MultihashInspect,

    // ---------------- Ceramic Tools ----------------------------//
    /// Create a stream ID
    #[cfg(feature = "ceramic")]
    StreamIdCreate(StreamIdCreateArgs),
    /// Inspect a stream ID
    #[cfg(feature = "ceramic")]
    StreamIdInspect(StreamIdInspectArgs),
    /// Generate a random stream ID
    #[cfg(feature = "ceramic")]
    StreamIdGenerate(StreamIdGenerateArgs),
    /// Construct a stream ID from raw bytes
    #[cfg(feature = "ceramic")]
    StreamIdFromBytes,
    /// Generate a random event ID
    #[cfg(feature = "ceramic")]
    EventIdGenerate(EventIdGenerateArgs),
    /// Inspect a multibase encoded event ID
    #[cfg(feature = "ceramic")]
    EventIdInspect(EventIdInspectArgs),
    /// Inspect an event car file
    #[cfg(feature = "ceramic")]
    EventInspect,
    /// Inspect a multibase encoded interest
    #[cfg(feature = "ceramic")]
    InterestInspect(InterestInspectArgs),
    /// Generate a random did:key method
    #[cfg(feature = "ceramic")]
    DidKeyGenerate,
    /// Generate a random peer ID
    #[cfg(feature = "ceramic")]
    PeerIdGenerate,

    // ---------------- IPLD Tools ----------------------------//
    /// Generate a random stream ID
    #[cfg(feature = "ipld")]
    CidGenerate,
    /// Output CID as raw bytes
    #[cfg(feature = "ipld")]
    CidAsBytes(CidInspectArgs),
    /// Inspect a CID
    #[cfg(feature = "ipld")]
    CidInspect(CidInspectArgs),
    /// Construct a CID from CID bytes
    #[cfg(feature = "ipld")]
    CidFromBytes,
    /// Hash bytes to compute a CID
    #[cfg(feature = "ipld")]
    CidFromData(CidFromDataArgs),
    /// Convert DAG-JSON data to DAG-CBOR
    #[cfg(feature = "ipld")]
    DagJsonToCbor,
    /// Convert DAG-CBOR data to DAG-JSON
    #[cfg(feature = "ipld")]
    DagCborToJson,
    /// Convert DAG-JOSE data to DAG-JSON
    #[cfg(feature = "ipld")]
    DagJoseToJson,
    /// Inspect DAG-CBOR encoded data
    #[cfg(feature = "ipld")]
    DagCborInspect,
    /// Index into DAG-CBOR encoded data
    #[cfg(feature = "ipld")]
    DagCborIndex(DagCborIndexArgs),
    /// List contents of a CAR archive
    #[cfg(feature = "ipld")]
    CarInspect(CarInspectArgs),
    /// Extract a single root CID from a CAR archive
    #[cfg(feature = "ipld")]
    CarExtract(CarExtractArgs),
    /// Construct a CAR file bytes from a list of blocks
    #[cfg(feature = "ipld")]
    CarFromBlocks(CarFromBlocksArgs),
    /// Deconstruct a CAR into its constituent blocks
    #[cfg(feature = "ipld")]
    CarToBlocks(CarToBlocksArgs),

    // ---------------- Libp2p Tools ----------------------------//
    /// Ping a peer
    #[cfg(feature = "p2p")]
    P2pPing(PingArgs),
    /// Contact a peer and negitiate and identify exchange
    #[cfg(feature = "p2p")]
    P2pIdentify(IdentifyArgs),

    // ---------------- Parquet Tools ----------------------------//
    /// Dump the content of parquet files with the give format
    #[cfg(feature = "parquet")]
    ParquetDump(ParquetDump),
    /// Inspect the metadata of a parquet file
    #[cfg(feature = "parquet")]
    ParquetInspect(ParquetInspect),
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

#[derive(Args, Debug, Clone)]
pub struct CarToBlocksArgs {
    /// Path to save the extracted blocks (default to current directory)
    #[arg(default_value = ".")]
    pub blocks_dir: String,
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

#[derive(Args, Debug, Clone)]
pub struct ParquetDump {
    /// Paths of parquet files to pretty format.
    /// It is assumed all files have the same schema.
    #[arg()]
    pub files: Vec<String>,

    #[arg(long, default_value_t = PrintFormat::Pretty, value_enum)]
    pub format: PrintFormat,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum PrintFormat {
    Csv,
    Json,
    Pretty,
}

#[derive(Args, Debug, Clone)]
pub struct ParquetInspect {
    /// Path of a parquet file to inspect.
    #[arg()]
    pub file: String,
}
