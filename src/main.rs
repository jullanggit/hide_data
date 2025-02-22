#![feature(bstr)]
#![feature(array_windows)]
#![feature(let_chains)]

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
    Normal,
    /// Print as numbers
    Bytes,
    /// Write to `path`
    File {
        path: PathBuf,
    },
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
                DecodeType::Normal => println!("{}", decode(base)),
                DecodeType::Bytes => {
                    for byte in decode(base).0 {
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

            for byte in char.encode_utf8(&mut [0; 4]).as_bytes() {
                base.insert(index, *byte);
                index += 1;
            }
        }

        if index < base.len() {
            index += 1
        };
    }
}

// TODO: Be smart about interleaving len
fn decode(bstr: &ByteStr) -> ByteString {
    let string = String::from_utf8_lossy(&bstr.0);

    let mut out = ByteString(Vec::new());

    for char in string.chars() {
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
