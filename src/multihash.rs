use anyhow::Result;
use futures::pin_mut;
use multihash::Multihash;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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

pub async fn run(op: Operation, stdin: impl AsyncRead, stdout: impl AsyncWrite) -> Result<()> {
    pin_mut!(stdin, stdout);
    match op {
        Operation::MultihashInspect => {
            let mut bytes = Vec::with_capacity(1024);
            stdin.read_to_end(&mut bytes).await?;
            let hash: Multihash<32> = Multihash::from_bytes(&bytes)?;
            stdout
                .write_all(
                    format!(
                        "Code: {}\nSize: {}\nDigest(hex): {}\n",
                        hash.code(),
                        hash.size(),
                        hex::encode(hash.digest())
                    )
                    .as_bytes(),
                )
                .await?;
        }
    };
    Ok(())
}
