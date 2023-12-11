use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = ceramic_pocket_knife::Cli::parse();
    ceramic_pocket_knife::run(args, tokio::io::stdin(), tokio::io::stdout()).await
}
