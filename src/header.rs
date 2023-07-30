use bytes::{Buf, Bytes};
use thiserror::Error;

use crate::util;

pub const CODEC_MAGIC: u32 = 0x3fd76c17;

pub fn check_magic(bytes: &mut Bytes) -> Result<u32, HeaderError> {
    let expected = bytes.get_u32();

    match expected {
        CODEC_MAGIC => Ok(expected),
        _ => Err(HeaderError::MagicMismatch {
            expected,
            actual: CODEC_MAGIC,
        }),
    }
}

pub fn check_header(
    bytes: &mut Bytes,
    codec: &str,
    min_version: u32,
    max_version: u32,
) -> Result<u32, HeaderError> {
    let actual = bytes.get_u32();

    println!("Magic - {}", actual);

    match actual {
        CODEC_MAGIC => check_header_no_magic(bytes, codec, min_version, max_version),
        _ => Err(HeaderError::MagicMismatch {
            actual,
            expected: CODEC_MAGIC,
        }),
    }
}

pub fn check_header_no_magic(
    bytes: &mut Bytes,
    codec: &str,
    min_version: u32,
    max_version: u32,
) -> Result<u32, HeaderError> {
    let actual = util::read_string(bytes)?;

    println!("Codec - {}", actual);

    if actual != codec {
        return Err(HeaderError::CodecMismatch {
            actual,
            expected: codec.to_string(),
        });
    }

    let version = bytes.get_u32();

    println!("version - {}", version);

    match version {
        v if v < min_version => Err(HeaderError::VersionTooOld),
        v if v > max_version => Err(HeaderError::VersionTooNew),
        _ => Ok(version),
    }
}

pub fn check_header_id(bytes: &mut Bytes, id: &Vec<u8>) -> Result<Vec<u8>, HeaderError> {
    let actual = util::read_id(bytes);

    if &actual == id {
        return Ok(actual);
    } else {
        return Err(HeaderError::BadHeaderId);
    }
}

pub fn check_header_suffix(bytes: &mut Bytes, suffix: &str) -> Result<String, HeaderError> {
    let suffix_length = bytes.get_u8();

    let actual = util::read_string_fixed(bytes, suffix_length as usize)?;

    if actual == suffix {
        return Ok(actual);
    }

    Err(HeaderError::BadHeaderSuffix)
}

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("Magic does not match")]
    MagicMismatch { expected: u32, actual: u32 },
    #[error("Codec does not match")]
    CodecMismatch { expected: String, actual: String },
    #[error("Malformed codec")]
    MalformedCodec(#[from] std::string::FromUtf8Error),
    #[error("Version is older than minimum")]
    VersionTooOld,
    #[error("Version is newer than supported")]
    VersionTooNew,
    #[error("Bad header id")]
    BadHeaderId,
    #[error("Bad header suffix")]
    BadHeaderSuffix,
}
