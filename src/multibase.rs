use anyhow::Result;
use futures::pin_mut;
use multibase::Base;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::cli::Command;

pub enum Operation {
    Guess,
    Decode,
    Base2,
    Base8,
    Base10,
    Base16,
    Base16Upper,
    Base32Hex,
    Base32HexUpper,
    Base32,
    Base32Upper,
    Base32Z,
    Base36,
    Base36Upper,
    Base58Flickr,
    Base58Btc,
    Base64,
    Base64Url,
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::BaseGuess => Ok(Operation::Guess),
            Command::BaseDecode => Ok(Operation::Decode),
            Command::Base2 => Ok(Operation::Base2),
            Command::Base8 => Ok(Operation::Base8),
            Command::Base10 => Ok(Operation::Base10),
            Command::Base16 => Ok(Operation::Base16),
            Command::Base16Upper => Ok(Operation::Base16Upper),
            Command::Base32Hex => Ok(Operation::Base32Hex),
            Command::Base32HexUpper => Ok(Operation::Base32HexUpper),
            Command::Base32 => Ok(Operation::Base32),
            Command::Base32Upper => Ok(Operation::Base32Upper),
            Command::Base32Z => Ok(Operation::Base32Z),
            Command::Base36 => Ok(Operation::Base36),
            Command::Base36Upper => Ok(Operation::Base36Upper),
            Command::Base58Flickr => Ok(Operation::Base58Flickr),
            Command::Base58Btc => Ok(Operation::Base58Btc),
            Command::Base64 => Ok(Operation::Base64),
            Command::Base64Url => Ok(Operation::Base64Url),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation, stdin: impl AsyncRead, stdout: impl AsyncWrite) -> Result<()> {
    pin_mut!(stdout);
    match op {
        Operation::Guess => {
            if let Some((base, is_multibase)) = guess(input_utf8(stdin).await?.trim_end()) {
                stdout
                    .write_all(format!("{:?} is_multibase: {}", base, is_multibase).as_bytes())
                    .await?
            }
        }
        Operation::Decode => {
            stdout
                .write_all(&multibase::decode(input_utf8(stdin).await?.trim_end())?.1)
                .await?
        }
        Operation::Base2 => encode(stdin, stdout, Base::Base2).await?,
        Operation::Base8 => encode(stdin, stdout, Base::Base8).await?,
        Operation::Base10 => encode(stdin, stdout, Base::Base10).await?,
        Operation::Base16 => encode(stdin, stdout, Base::Base16Lower).await?,
        Operation::Base16Upper => encode(stdin, stdout, Base::Base16Upper).await?,
        Operation::Base32Hex => encode(stdin, stdout, Base::Base32HexLower).await?,
        Operation::Base32HexUpper => encode(stdin, stdout, Base::Base32HexUpper).await?,
        Operation::Base32 => encode(stdin, stdout, Base::Base32Lower).await?,
        Operation::Base32Upper => encode(stdin, stdout, Base::Base32Upper).await?,
        Operation::Base32Z => encode(stdin, stdout, Base::Base32Z).await?,
        Operation::Base36 => encode(stdin, stdout, Base::Base36Lower).await?,
        Operation::Base36Upper => encode(stdin, stdout, Base::Base36Upper).await?,
        Operation::Base58Flickr => encode(stdin, stdout, Base::Base58Flickr).await?,
        Operation::Base58Btc => encode(stdin, stdout, Base::Base58Btc).await?,
        Operation::Base64 => encode(stdin, stdout, Base::Base64).await?,
        Operation::Base64Url => encode(stdin, stdout, Base::Base64Url).await?,
    };
    Ok(())
}

async fn input_bytes(stdin: impl AsyncRead) -> Result<Vec<u8>> {
    pin_mut!(stdin);
    let mut data = Vec::new();
    stdin.read_to_end(&mut data).await?;
    Ok(data)
}
async fn input_utf8(stdin: impl AsyncRead) -> Result<String> {
    pin_mut!(stdin);
    let mut data = Vec::new();
    stdin.read_to_end(&mut data).await?;
    Ok(String::from_utf8(data)?)
}

async fn encode(stdin: impl AsyncRead, stdout: impl AsyncWrite, base: Base) -> Result<()> {
    pin_mut!(stdout);
    stdout
        .write_all(multibase::encode(base, input_bytes(stdin).await?).as_bytes())
        .await?;
    Ok(())
}

fn guess(data: &str) -> Option<(Base, bool)> {
    // First try to decode as a valid multibase
    if let Ok((base, _)) = multibase::decode(data) {
        return Some((base, true));
    };
    let bases = vec![
        Base::Base2,
        Base::Base8,
        Base::Base10,
        Base::Base16Lower,
        Base::Base16Upper,
        Base::Base32HexLower,
        Base::Base32HexUpper,
        Base::Base32Lower,
        Base::Base32Upper,
        Base::Base32Z,
        Base::Base58Flickr,
        Base::Base58Btc,
        Base::Base64,
        Base::Base64Url,
    ];
    for base in bases {
        if let Ok(_res) = base.decode(data) {
            return Some((base, false));
        }
    }
    None
}
