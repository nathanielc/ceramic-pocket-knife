#[cfg(feature = "ceramic")]
mod ceramic;
#[cfg(feature = "ipld")]
mod ipld;
#[cfg(feature = "multibase")]
mod multibase;
#[cfg(feature = "multihash")]
mod multihash;
#[cfg(feature = "p2p")]
mod p2p;

pub mod cli;

pub use cli::Cli;

use clap::CommandFactory;
use tokio::io::{AsyncRead, AsyncWrite};

pub async fn run(args: Cli, stdin: impl AsyncRead, stdout: impl AsyncWrite) -> anyhow::Result<()> {
    // Generate shell completetions
    if let cli::Command::Completion(args) = &args.command {
        args.shell
            // TODO: use async stdout
            .generate(&mut Cli::command(), &mut std::io::stdout());
        return Ok(());
    };

    // Try each category of command in turn, until we find a match.
    #[allow(unused)]
    let cmd = args.command;

    #[cfg(feature = "multibase")]
    #[allow(unused)]
    let cmd = match multibase::Operation::try_from(cmd) {
        Ok(op) => {
            return multibase::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };

    #[cfg(feature = "ipld")]
    #[allow(unused)]
    let cmd = match ipld::Operation::try_from(cmd) {
        Ok(op) => {
            return ipld::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };

    #[cfg(feature = "ceramic")]
    #[allow(unused)]
    let cmd = match ceramic::Operation::try_from(cmd) {
        Ok(op) => {
            return ceramic::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };

    #[cfg(feature = "multihash")]
    #[allow(unused)]
    let cmd = match multihash::Operation::try_from(cmd) {
        Ok(op) => {
            return multihash::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };

    #[cfg(feature = "p2p")]
    #[allow(unused)]
    let cmd = match p2p::Operation::try_from(cmd) {
        Ok(op) => {
            return p2p::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };
    Err(anyhow::anyhow!("failed to match command"))
}

#[cfg(any(feature = "ipld", feature = "ceramic"))]
fn random_cid() -> cid::Cid {
    let mut data = [0u8; 8];
    rand::Rng::fill(&mut rand::thread_rng(), &mut data);
    let hash = ::multihash::MultihashDigest::digest(&::multihash::Code::Sha2_256, &data);
    cid::Cid::new_v1(0x00, hash)
}
