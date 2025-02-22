use clap::{Args, Parser, Subcommand};
use std::{mem::MaybeUninit, path::PathBuf};

#[derive(Parser)]
struct Cli {
    #[arg(value_parser = clap::builder::NonEmptyStringValueParser::new())]
    base: String,
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
    String { hide: String },
    /// Hides `length` random bytes in base
    Random { length: usize },
    /// Hides the contents of `path` in base
    File { path: PathBuf },
}

#[derive(Args, Debug)]
struct Decode {
    #[command(subcommand)]
    decode_type: DecodeType,
}

#[derive(Subcommand, Debug)]
enum DecodeType {
    /// Prints as string
    String,
    /// Prints as lossy string
    StringLossy,
    /// Prints as numbers
    Bytes,
    /// Unhides the hidden bytes from `base` into `path`
    File { path: PathBuf },
}

fn main() {
    use Mode::{Decode, Encode};

    let args = Cli::parse();
    let base = &args.base.trim();

    match args.mode {
        Encode(to_encode) => match to_encode.encode_type {
            EncodeType::String { hide } => println!("{}", encode(base, hide.as_bytes())),
            EncodeType::Random { length } => {
                let mut buf = vec![0; length];
                getrandom::fill(&mut buf).unwrap();

                println!("{}", encode(base, &buf));
            }
            EncodeType::File { path } => todo!(),
        },
        Decode(to_decode) => match to_decode.decode_type {
            DecodeType::String => println!("{}", str::from_utf8(&decode(base)).unwrap()),
            DecodeType::StringLossy => {
                println!("{}", String::from_utf8_lossy(&decode(base)))
            }
            DecodeType::Bytes => {
                for byte in decode(base) {
                    print!("{byte} ");
                }
            }
            DecodeType::File { path } => todo!(),
        },
    }
}

/// Hides `hide` in `base`
fn encode(base: &str, hide: &[u8]) -> String {
    let hide_per_char = (hide.len() / base.len()).max(1);
    let mut hide_chunks = hide.chunks(hide_per_char);

    let mut out = String::new();

    for char in base.chars() {
        out.push(char);

        if let Some(chunk) = hide_chunks.next() {
            for byte in chunk {
                out.push(byte_to_variation_selector(*byte));
            }
        }
    }

    // Handle remainder
    if let Some(chunk) = hide_chunks.next() {
        for byte in chunk {
            out.push(byte_to_variation_selector(*byte));
        }
    }

    out
}

// TODO: Be smart about interleaving len
fn decode(str: &str) -> Vec<u8> {
    let mut out = Vec::new();

    for char in str.chars() {
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
