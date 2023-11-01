
use anyhow::Result;
use tokio::io::{self, AsyncReadExt};

use crate::cli::Command;

pub enum Operation {
    MultihashInspect,
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::MultihashInspect => Ok(Operation::MultihashInspect),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation) -> Result<()> {
    match op {
        Operation::MultihashInspect => {
            let mut bytes = Vec::with_capacity(1024);
            io::stdin().read_to_end(&mut bytes).await?;
            let hash = multihash::Multihash::from_bytes(&bytes)?;
            println!(
                "Code: {}\nSize: {}\nDigest(hex): {}",
                hash.code(),
                hash.size(),
                hex::encode(hash.digest())
            );
        }
    };
    Ok(())
}
