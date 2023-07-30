use bytes::{Buf, Bytes};

use super::super::SegmentInfo;
use crate::directory_reader::DirectoryReader;

/// Lucene 9.0 Segment Info Format.
pub struct Lucene90SegmentInfoFormat {}

impl Lucene90SegmentInfoFormat {
    pub const SI_EXTENSION: &'static str = "si";

    /// Reads segment info from the disk
    pub fn read(directory_reader: &DirectoryReader, name: String, id: Vec<u8>) -> SegmentInfo {
        let file_name = format!("{}.{}", name, Self::SI_EXTENSION);

        let mut bytes = directory_reader.read_file(&file_name);

        Self::check_header(&mut bytes, &id);

        // Parse segment info data

        return SegmentInfo { name, id };
    }

    fn check_header(bytes: &mut Bytes, id: &Vec<u8>) {}

    fn getMinVersion(bytes: &mut Bytes) -> MinVersion {
        let has_min_version = bytes.get_u8();

        match has_min_version {
            0 => MinVersion::None,
            1 => MinVersion::Some((bytes.get_u32(), bytes.get_u32(), bytes.get_u32())),
            _ => MinVersion::Invalid(has_min_version),
        }
    }
}

enum MinVersion {
    None,
    Some((u32, u32, u32)),
    Invalid(u8),
}
