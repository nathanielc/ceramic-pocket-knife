mod ceramic;
pub mod cli;
mod ipld;
mod multibase;
mod multihash;
mod p2p;

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
    let cmd = match multibase::Operation::try_from(args.command) {
        Ok(op) => {
            return multibase::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };
    let cmd = match ipld::Operation::try_from(cmd) {
        Ok(op) => {
            return ipld::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };
    let cmd = match ceramic::Operation::try_from(cmd) {
        Ok(op) => {
            return ceramic::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };
    let cmd = match multihash::Operation::try_from(cmd) {
        Ok(op) => {
            return multihash::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };
    let _cmd = match p2p::Operation::try_from(cmd) {
        Ok(op) => {
            return p2p::run(op, stdin, stdout).await;
        }
        Err(cmd) => cmd,
    };
    Err(anyhow::anyhow!("failed to match command"))
}
