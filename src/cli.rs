use std::path::PathBuf;

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

    // ---------------- Ceramic Tools ----------------------------//
    /// Generate a random stream ID
    CidGenerate,
    /// Create a stream ID
    StreamIdCreate(CreateStreamIdArgs),
    /// Inspect a stream ID
    StreamIdInspect(InspectStreamIdArgs),
    /// Generate a random stream ID
    StreamIdGenerate(GenerateStreamIdArgs),
    /// Generate a random event ID
    EventIdGenerate(GenerateEventIdArgs),
    /// Decode a hex encoded event ID
    EventIdDecode(DecodeEventIdArgs),
    /// Decode a hex encoded interest
    InterestDecode(DecodeInterestArgs),
    /// Generate a random did:key method
    DidKeyGenerate,
    /// Generate a random peer ID
    PeerIdGenerate,
    /// Generate a ceramic-one sqldb with random data
    SqlDbGenerate(GenerateSqlDbArgs),
}

#[derive(Args, Debug, Clone)]
pub struct CompletionArgs {
    /// Shell type.
    #[arg(value_enum)]
    pub shell: clap_complete_command::Shell,
}

#[derive(Args, Debug, Clone)]
pub struct CreateStreamIdArgs {
    /// Stream type.
    #[arg(long, value_enum)]
    pub r#type: StreamType,
    /// Init CID of the stream
    #[arg(long)]
    pub cid: String,
}

#[derive(Args, Debug, Clone)]
pub struct InspectStreamIdArgs {
    /// Stream ID
    #[arg()]
    pub id: String,
}

#[derive(Args, Debug, Clone)]
pub struct GenerateStreamIdArgs {
    /// Stream type.
    #[arg(long, value_enum)]
    pub r#type: StreamType,
}

#[derive(Args, Debug, Clone)]
pub struct GenerateEventIdArgs {
    /// Network
    #[arg(long, default_value = "testnet-clay", value_enum)]
    pub network: Network,
    /// Sort Key, if not set generates random value.
    #[arg(long)]
    pub sort_key: Option<String>,
    /// Sort Value, if not set generates random value.
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
pub struct DecodeEventIdArgs {
    /// Hex encoded Event ID to decode
    #[arg()]
    pub event_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct DecodeInterestArgs {
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
pub struct GenerateSqlDbArgs {
    /// Network
    #[arg(long, default_value = "./ceramic-one.db")]
    pub path: PathBuf,
    /// Network
    #[arg(long, default_value = "100")]
    pub count: usize,
    /// Network
    #[arg(long, default_value = "testnet-clay", value_enum)]
    pub network: Network,
    /// Sort Key for all events.
    #[arg(long, default_value = "model")]
    pub sort_key: String,
    /// Sort Value, if not set generates random value per event.
    #[arg(long)]
    pub sort_value: Option<String>,
    /// Controller, if not set generates random value per event.
    #[arg(long)]
    pub controller: Option<String>,
    /// Stream ID of init event, if not set generates random value per event.
    #[arg(long)]
    pub init_id: Option<String>,
}
