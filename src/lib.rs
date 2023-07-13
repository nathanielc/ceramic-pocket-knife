use clap::Parser;

mod ceramic;
mod cli;
mod multibase;

pub use cli::Cli;

pub fn run() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    // Try each category of command in turn, until we

    let cmd = match multibase::Operation::try_from(args.command) {
        Ok(op) => {
            return multibase::run(op);
        }
        Err(cmd) => cmd,
    };
    let _cmd = match ceramic::Operation::try_from(cmd) {
        Ok(op) => {
            return ceramic::run(op);
        }
        Err(cmd) => cmd,
    };
    Err(anyhow::anyhow!("failed to match command"))
}
