use anyhow::Result;
use multibase::Base;
use std::io::{self, Write as _};

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
            Command::Base58Flickr => Ok(Operation::Base58Flickr),
            Command::Base58Btc => Ok(Operation::Base58Btc),
            Command::Base64 => Ok(Operation::Base64),
            Command::Base64Url => Ok(Operation::Base64Url),
            _ => Err(value),
        }
    }
}

pub fn run(op: Operation) -> Result<()> {
    let lines = io::stdin().lines();
    for line in lines {
        do_line(&line?, &op)?;
    }
    Ok(())
}

fn do_line(line: &str, op: &Operation) -> Result<()> {
    let data: Vec<u8> = match op {
        Operation::Guess => {
            if let Some((base, is_multibase)) = guess(line) {
                format!("{:?} is_multibase: {}", base, is_multibase)
                    .as_bytes()
                    .to_vec()
            } else {
                Vec::default()
            }
        }
        Operation::Decode => multibase::decode(line)?.1,
        Operation::Base2 => encode(line, Base::Base2)?,
        Operation::Base8 => encode(line, Base::Base8)?,
        Operation::Base10 => encode(line, Base::Base10)?,
        Operation::Base16 => encode(line, Base::Base16Lower)?,
        Operation::Base16Upper => encode(line, Base::Base16Upper)?,
        Operation::Base32Hex => encode(line, Base::Base32HexLower)?,
        Operation::Base32HexUpper => encode(line, Base::Base32HexUpper)?,
        Operation::Base32 => encode(line, Base::Base32Lower)?,
        Operation::Base32Upper => encode(line, Base::Base32Upper)?,
        Operation::Base32Z => encode(line, Base::Base32Z)?,
        Operation::Base58Flickr => encode(line, Base::Base58Flickr)?,
        Operation::Base58Btc => encode(line, Base::Base58Btc)?,
        Operation::Base64 => encode(line, Base::Base64)?,
        Operation::Base64Url => encode(line, Base::Base64Url)?,
    };
    std::io::stdout().write_all(&data)?;
    println!();
    Ok(())
}

fn encode(line: &str, base: Base) -> Result<Vec<u8>> {
    Ok(multibase::encode(base, line.as_bytes()).into())
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
