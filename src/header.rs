use bytes::{Buf, Bytes};
use thiserror::Error;

use crate::util::read_variable_string;

pub const CODEC_MAGIC: u32 = 0x3fd76c17;

pub const MIN_VERSION: u32 = 9;
pub const CURRENT_VERSION: u32 = 10;

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

pub fn check_header(bytes: &mut Bytes, codec: &str) -> Result<u32, HeaderError> {
    let expected = bytes.get_u32();

    match expected {
        CODEC_MAGIC => check_header_no_magic(bytes, codec),
        _ => Err(HeaderError::MagicMismatch {
            expected,
            actual: CODEC_MAGIC,
        }),
    }
}

pub fn check_header_no_magic(bytes: &mut Bytes, codec: &str) -> Result<u32, HeaderError> {
    let expected_codec = read_variable_string(bytes)?;

    if expected_codec != codec {
        return Err(HeaderError::CodecMismatch {
            expected: expected_codec,
            actual: codec.to_string(),
        });
    }

    let version = bytes.get_u32();

    match version {
        v if v < MIN_VERSION => Err(HeaderError::VersionTooOld),
        v if v > CURRENT_VERSION => Err(HeaderError::VersionTooNew),
        _ => Ok(version),
    }
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
}
