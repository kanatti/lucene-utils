use bytes::{Buf, Bytes};
use thiserror::Error;

use super::super::SegmentInfo;
use crate::{
    directory_reader::DirectoryReader,
    header::{self, HeaderError},
    util,
};

/// Lucene 9.0 Segment Info Format.
pub struct Lucene90SegmentInfoFormat {}

impl Lucene90SegmentInfoFormat {
    pub const SI_EXTENSION: &'static str = "si";
    pub const CODEC: &'static str = "Lucene90SegmentInfo";
    pub const VERSION_START: u32 = 0;
    pub const VERSION_CURRENT: u32 = Self::VERSION_START;

    /// Reads segment info from the disk
    pub fn read(
        directory_reader: &DirectoryReader,
        name: String,
        id: Vec<u8>,
    ) -> Result<SegmentInfo, SegmentInfoReadError> {
        let file_name = format!("{}.{}", name, Self::SI_EXTENSION);

        let mut bytes = directory_reader.read_file(&file_name);

        println!("bytes {:?}", bytes.to_vec());

        // Check different parts of the header

        header::check_header(
            &mut bytes,
            Self::CODEC,
            Self::VERSION_START,
            Self::VERSION_CURRENT,
        )?;

        // Check Index Header ID
        header::check_header_id(&mut bytes, &id)?;

        // Check Index Header suffix
        header::check_header_suffix(&mut bytes, "")?;

        // Parse segment info data

        let version = (bytes.get_u32_le(), bytes.get_u32_le(), bytes.get_u32_le());

        println!("version {:?}", version);

        let min_version = Self::get_min_version(&mut bytes)?;
        let doc_count = bytes.get_u32_le();
        let is_compound_file = bytes.get_u8() == 1;

        let diagnostics = util::read_map(&mut bytes);
        let files = util::read_set(&mut bytes);
        let attributes = util::read_map(&mut bytes);
        let sort_fields = util::read_vec(&mut bytes);

        return Ok(SegmentInfo {
            name,
            id,
            version,
            min_version,
            doc_count,
            is_compound_file,
            diagnostics,
            files,
            attributes,
            sort_fields
        });
    }

    fn get_min_version(bytes: &mut Bytes) -> Result<Option<(u32, u32, u32)>, SegmentInfoReadError> {
        let has_min_version = bytes.get_u8();

        match has_min_version {
            0 => Ok(None),
            1 => Ok(Some((
                bytes.get_u32_le(),
                bytes.get_u32_le(),
                bytes.get_u32_le(),
            ))),
            _ => Err(SegmentInfoReadError::CorruptSegmentError(format!(
                "Bad has_min_version - {}",
                has_min_version
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum SegmentInfoReadError {
    #[error("header error")]
    DummyError(#[from] HeaderError),
    #[error("corrupt segment error")]
    CorruptSegmentError(String),
}
