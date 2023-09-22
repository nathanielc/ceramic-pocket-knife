use anyhow::Result;
use multibase::Base;
use std::io::{stdin, stdout, Read, Write};

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

pub fn run(op: Operation) -> Result<()> {
    match op {
        Operation::Guess => {
            if let Some((base, is_multibase)) = guess(input_utf8()?.trim_end()) {
                println!("{:?} is_multibase: {}", base, is_multibase)
            }
        }
        Operation::Decode => stdout().write_all(&multibase::decode(input_utf8()?.trim_end())?.1)?,
        Operation::Base2 => encode(Base::Base2)?,
        Operation::Base8 => encode(Base::Base8)?,
        Operation::Base10 => encode(Base::Base10)?,
        Operation::Base16 => encode(Base::Base16Lower)?,
        Operation::Base16Upper => encode(Base::Base16Upper)?,
        Operation::Base32Hex => encode(Base::Base32HexLower)?,
        Operation::Base32HexUpper => encode(Base::Base32HexUpper)?,
        Operation::Base32 => encode(Base::Base32Lower)?,
        Operation::Base32Upper => encode(Base::Base32Upper)?,
        Operation::Base32Z => encode(Base::Base32Z)?,
        Operation::Base36 => encode(Base::Base36Lower)?,
        Operation::Base36Upper => encode(Base::Base36Upper)?,
        Operation::Base58Flickr => encode(Base::Base58Flickr)?,
        Operation::Base58Btc => encode(Base::Base58Btc)?,
        Operation::Base64 => encode(Base::Base64)?,
        Operation::Base64Url => encode(Base::Base64Url)?,
    };
    Ok(())
}

fn input_bytes() -> Result<Vec<u8>> {
    let mut data = Vec::new();
    stdin().read_to_end(&mut data)?;
    Ok(data)
}
fn input_utf8() -> Result<String> {
    let mut data = Vec::new();
    stdin().read_to_end(&mut data)?;
    Ok(String::from_utf8(data)?)
}

fn encode(base: Base) -> Result<()> {
    println!("{}", multibase::encode(base, input_bytes()?));
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
