mod ceramic;
mod cli;
mod ipld;
mod multibase;
mod multihash;
mod p2p;

pub use cli::Cli;

use clap::{CommandFactory, Parser};

pub async fn run() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    // Generate shell completetions
    if let cli::Command::Completion(args) = &args.command {
        args.shell
            .generate(&mut Cli::command(), &mut std::io::stdout());
        return Ok(());
    };

    // Try each category of command in turn, until we find a match.
    let cmd = match multibase::Operation::try_from(args.command) {
        Ok(op) => {
            return multibase::run(op);
        }
        Err(cmd) => cmd,
    };
    let cmd = match ipld::Operation::try_from(cmd) {
        Ok(op) => {
            return ipld::run(op).await;
        }
        Err(cmd) => cmd,
    };
    let cmd = match ceramic::Operation::try_from(cmd) {
        Ok(op) => {
            return ceramic::run(op).await;
        }
        Err(cmd) => cmd,
    };
    let cmd = match multihash::Operation::try_from(cmd) {
        Ok(op) => {
            return multihash::run(op).await;
        }
        Err(cmd) => cmd,
    };
    let _cmd = match p2p::Operation::try_from(cmd) {
        Ok(op) => {
            return p2p::run(op).await;
        }
        Err(cmd) => cmd,
    };
    Err(anyhow::anyhow!("failed to match command"))
}
