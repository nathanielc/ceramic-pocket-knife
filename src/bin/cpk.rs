#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    ceramic_pocket_knife::run().await
}
