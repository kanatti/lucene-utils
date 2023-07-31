use bytes::Buf;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

mod directory_reader;
mod header;
mod lucene_version;
mod segment_info;
mod util;

use header::HeaderError;
use segment_info::format::Lucene90SegmentInfoFormat;

use directory_reader::DirectoryReader;
use directory_reader::IndexInput;

use crate::segment_info::SegmentCommitInfo;

pub const SEGMENT_FILE_NAME: &'static str = "segments";
pub const SEGMENTS_CODEC: &'static str = "segments";

pub fn load_index(index_path: &Path) {
    let index_files: Vec<PathBuf> = std::fs::read_dir(index_path)
        .expect("Cant read dir")
        .filter(|entry| entry.is_ok())
        .filter(|entry| entry.as_ref().unwrap().path().is_file())
        .map(|entry| entry.unwrap().path())
        .collect();

    for index_file in &index_files {
        println!("{:?}", index_file);
    }

    match get_last_commit_generation(&index_files) {
        Some(generation) => {
            println!("Generation: {}", generation);

            let segment_infos = SegmentInfos::read_commit(index_path, generation);
            println!("{:#?}", segment_infos);
        }
        _ => {
            println!("No commit found");
        }
    }
}

//// Gets the last commit generation from a list of segment files.
pub fn get_last_commit_generation<'a>(file_paths: &[PathBuf]) -> Option<u32> {
    file_paths
        .iter()
        .map(|path| path.file_name().unwrap().to_str().unwrap())
        .filter(|file_name| file_name.starts_with(SEGMENT_FILE_NAME))
        .map(|segment_file_name| get_generation_from_segments_file_name(segment_file_name))
        .max()
}

pub fn get_generation_from_segments_file_name(segments_file_name: &str) -> u32 {
    match segments_file_name {
        SEGMENT_FILE_NAME => 0,
        _ => segments_file_name[(SEGMENT_FILE_NAME.len() + 1)..]
            .parse::<u32>()
            .unwrap(),
    }
}

pub fn get_segment_file_name(gen: u32) -> String {
    match gen {
        0 => SEGMENT_FILE_NAME.to_string(),
        n => format!("{}_{}", SEGMENT_FILE_NAME, n),
    }
}

#[derive(Debug)]
pub struct SegmentInfos {
    pub version: u64,
    pub index_created_version: u8,
    pub generation: u32,
    pub lucene_version: (u8, u8, u8),
    pub id: Vec<u8>,
    pub counter: u64,
    pub num_segments: u32,
    pub min_segment_lucene_version: (u8, u8, u8),
    pub segments: Vec<SegmentCommitInfo>,
}

impl SegmentInfos {
    pub const MIN_VERSION: u32 = 9;
    pub const CURRENT_VERSION: u32 = 10;

    // TODO: Checksum for segment corruption
    pub fn read_commit(index_path: &Path, generation: u32) -> Result<Self, SegmentReadError> {
        let directory_reader = DirectoryReader { path: index_path };
        let mut index_input = directory_reader.open(&get_segment_file_name(generation));

        let _magic = header::check_magic(&mut index_input.bytes)?;

        let format_version = Self::check_header(&mut index_input)?;

        let id = index_input.read_id();

        println!("id: {:?}", id);

        Self::check_header_suffix(&mut index_input, &format!("{}", generation))?;

        // Fix: use vInt
        let lucene_version = (
            index_input.read_byte(),
            index_input.read_byte(),
            index_input.read_byte(),
        );

        let index_created_version = index_input.read_byte();

        println!("{:?}", lucene_version);
        println!("{}", index_created_version);

        if lucene_version.0 < index_created_version {
            return Err(SegmentReadError::CorruptedIndex);
        }

        if (index_created_version as u32) < Self::MIN_VERSION {
            return Err(SegmentReadError::IndexFormatTooOld);
        }

        let seg_info_version = index_input.read_long();

        // Fix: Use vLong
        let counter = index_input.read_byte() as u64;

        let num_segments = index_input.read_int();

        // Fix: Use vInt
        let min_segment_lucene_version = (
            index_input.read_byte(),
            index_input.read_byte(),
            index_input.read_byte(),
        );

        let mut segments = Vec::with_capacity(num_segments as usize);

        for _i in 0..num_segments {
            let segment_name = index_input.read_variable_string();
            let segment_id = index_input.read_id();

            // Fix: Add dynamic use of codec
            let codec_name = index_input.read_variable_string();

            println!("codec {}", codec_name);

            let segment_info =
                Lucene90SegmentInfoFormat::read(&directory_reader, segment_name, segment_id)
                    .unwrap();

            let del_gen = index_input.bytes.get_i64();

            let del_count = index_input.read_int();

            // TODO: Validate del_count and max_docs inside the segment info

            let field_infos_gen = index_input.bytes.get_i64();
            let dv_gen = index_input.bytes.get_i64();
            let soft_del_count = index_input.read_int();

            // TODO: Soft delete validations

            let sci_id = Self::read_segment_commit_info_id(format_version, &mut index_input)?;

            let field_info_files = util::read_set(&mut index_input.bytes);

            // TODO: Read DV Update Files
            let num_dv_fields = index_input.read_int();
            let dv_update_files : HashMap<u32, HashSet<String>> = HashMap::new();

            // TODO: Checks of segment info version and min_version

            let user_data = util::read_map(&mut index_input.bytes);

            let sci = SegmentCommitInfo {
                info: segment_info,
                del_count,
                soft_del_count,
                del_gen,
                field_infos_gen,
                dv_gen,
                sci_id,
                field_info_files,
                dv_update_files
            };

            segments.push(sci);

            println!("Remaining : {}", index_input.bytes.remaining()); // 16 remaining?
        }

        return Ok(SegmentInfos {
            version: seg_info_version,
            index_created_version,
            lucene_version,
            generation,
            id,
            counter,
            num_segments,
            min_segment_lucene_version,
            segments,
        });
    }

    /**
     * Checks header and returns version if all good
     */
    pub fn check_header(index_input: &mut IndexInput) -> Result<u32, SegmentReadError> {
        return header::check_header_no_magic(
            &mut index_input.bytes,
            SEGMENTS_CODEC,
            Self::MIN_VERSION,
            Self::CURRENT_VERSION,
        )
        .map_err(SegmentReadError::from);
    }

    pub fn check_header_suffix(
        index_input: &mut IndexInput,
        expected: &str,
    ) -> Result<(), SegmentReadError> {
        let suffix_len = index_input.read_byte() as usize;
        let suffix = index_input.read_string(suffix_len);

        if suffix != expected {
            return Err(SegmentReadError::CorruptedIndex);
        }

        Ok(())
    }

    pub fn read_segment_commit_info_id(
        format_version: u32,
        index_input: &mut IndexInput,
    ) -> Result<Option<Vec<u8>>, SegmentReadError> {
        match format_version {
            v if v > Self::MIN_VERSION => {
                let marker = index_input.read_byte();
                match marker {
                    1 => Ok(Some(index_input.read_id())),
                    0 => Ok(None),
                    _ => Err(SegmentReadError::CorruptedIndex),
                }
            }
            _ => Ok(None),
        }
    }
}

#[derive(Error, Debug)]
pub enum SegmentReadError {
    #[error("Index Format is too old")]
    IndexFormatTooOld,
    #[error("Index Format is too new")]
    IndexFormatTooNew,
    #[error("Index is corrupt")]
    CorruptedIndex,
}

impl From<header::HeaderError> for SegmentReadError {
    fn from(value: header::HeaderError) -> Self {
        match value {
            HeaderError::VersionTooOld => SegmentReadError::IndexFormatTooOld,
            HeaderError::VersionTooNew => SegmentReadError::IndexFormatTooNew,
            HeaderError::MagicMismatch {
                expected: _,
                actual: _,
            } => SegmentReadError::IndexFormatTooOld,
            _ => SegmentReadError::CorruptedIndex,
        }
    }
}
