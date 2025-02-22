#![feature(bstr)]
#![feature(array_chunks)]

use clap::{Args, Parser, Subcommand};
use std::{
    bstr::{ByteStr, ByteString},
    fs::{self},
    path::PathBuf,
};

#[derive(Parser)]
struct Cli {
    base: ByteString,
    #[command(subcommand)]
    mode: Mode,
}
#[derive(Subcommand, Debug)]
enum Mode {
    Encode(Encode),
    Decode(Decode),
}

#[derive(Args, Debug)]
struct Encode {
    #[command(subcommand)]
    encode_type: EncodeType,
}

#[derive(Subcommand, Debug)]
enum EncodeType {
    /// Hide the string `hide` in `base`
    Normal { hide: ByteString },
    /// Hides `length` random bytes in base
    Random { length: usize },
}

#[derive(Args, Debug)]
struct Decode {
    #[command(subcommand)]
    decode_type: DecodeType,
}

#[derive(Subcommand, Debug)]
enum DecodeType {
    /// Print as string
    String,
    /// Print as lossy string
    StringLossy,
    /// Print as numbers
    Bytes,
    /// Write to `path`
    File { path: PathBuf },
}

fn main() {
    use Mode::{Decode, Encode};

    let mut args = Cli::parse();

    match args.mode {
        Encode(to_encode) => {
            match to_encode.encode_type {
                EncodeType::Normal { hide } => encode(&mut args.base, &hide),
                EncodeType::Random { length } => {
                    let mut buf = vec![0; length];
                    getrandom::fill(&mut buf).unwrap();

                    encode(&mut args.base, &buf)
                }
            };
            println!("{}", args.base)
        }
        Decode(to_decode) => {
            let base = args.base.as_ref();
            match to_decode.decode_type {
                DecodeType::String => println!("{}", str::from_utf8(&decode(base)).unwrap()),
                DecodeType::StringLossy => {
                    println!("{}", String::from_utf8_lossy(&decode(base)))
                }
                DecodeType::Bytes => {
                    for byte in decode(base) {
                        print!("{byte} ");
                    }
                }
                DecodeType::File { path } => fs::write(path, decode(base)).unwrap(),
            };
        }
    }
}

/// Hides `hide` in `base`
fn encode(base: &mut ByteString, hide: &[u8]) {
    let hide_per_char = (hide.len() / base.len()).max(1);
    let hide_chunks = hide.chunks(hide_per_char);

    let mut index = 1; // After the first char
    for chunk in hide_chunks {
        for byte in chunk {
            let char = byte_to_variation_selector(*byte);
            let mut bytes = [0; 4];
            char.encode_utf8(&mut bytes);

            base.splice(index..index, bytes);

            index += 4;
        }

        index += 1;
    }
}

// TODO: Be smart about interleaving len
fn decode(str: &ByteStr) -> Vec<u8> {
    let mut out = Vec::new();

    for char in str.array_chunks() {
        let char = u32::from_ne_bytes(*char);

        if let Some(byte) = variant_selector_to_byte(char) {
            out.push(byte);
        }
    }

    out
}

fn byte_to_variation_selector(byte: u8) -> char {
    if byte < 16 {
        char::from_u32(0xFE00 + byte as u32).unwrap()
    } else {
        char::from_u32(0xE0100 + (byte - 16) as u32).unwrap()
    }
}

fn variant_selector_to_byte(char: char) -> Option<u8> {
    let encoded_byte = char as u32;

    if (0xFE00..0xFE0F).contains(&encoded_byte) {
        Some((encoded_byte - 0xFE00) as u8)
    } else if (0xE0100..0xE01EF).contains(&encoded_byte) {
        Some((encoded_byte - 0xE0100 + 16) as u8)
    } else {
        None
    }
}
