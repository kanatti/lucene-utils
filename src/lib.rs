use std::path::Path;
use std::path::PathBuf;

mod directory_reader;
mod lucene_version;

use directory_reader::DirectoryReader;
use directory_reader::IndexInput;

pub fn load_index(index_path: &Path) {
    // Load index files into an in-memory structure
    let index = Index::load(index_path).unwrap();
}

#[derive(Debug)]
pub struct Index {}

#[derive(Debug)]
pub enum LoadError {}

impl Index {
    // Load an index from disk
    pub fn load(index_path: &Path) -> Result<Index, LoadError> {
        // Get all file paths under the directory
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
                println!("{:?}", segment_infos);
            }
            _ => {
                println!("No commit found");
            }
        }

        Ok(Index {})
    }
}

pub struct IndexFileNames {}

impl IndexFileNames {
    pub const SEGMENT: &'static str = "segments";
}

//// Gets the last commit generation from a list of segment files.
pub fn get_last_commit_generation<'a>(file_paths: &[PathBuf]) -> Option<u32> {
    file_paths
        .iter()
        .map(|path| path.file_name().unwrap().to_str().unwrap())
        .filter(|file_name| file_name.starts_with(IndexFileNames::SEGMENT))
        .map(|segment_file_name| get_generation_from_segments_file_name(segment_file_name))
        .max()
}

pub fn get_generation_from_segments_file_name(segments_file_name: &str) -> u32 {
    match segments_file_name {
        IndexFileNames::SEGMENT => 0,
        _ => segments_file_name[(IndexFileNames::SEGMENT.len() + 1)..]
            .parse::<u32>()
            .unwrap(),
    }
}

pub fn get_segment_file_name(gen: u32) -> String {
    match gen {
        0 => IndexFileNames::SEGMENT.to_string(),
        n => format!("{}_{}", IndexFileNames::SEGMENT, n),
    }
}

pub struct Codec {}

impl Codec {
    pub const MAGIC: u32 = 0x3fd76c17;
    pub const SEGMENTS_CODEC: &'static str = "segments";
}

#[derive(Debug)]
pub struct SegmentInfos {
    pub index_created_version: u8,
    pub generation: u32,
    pub lucene_version: (u8, u8, u8),
    pub id: Vec<u8>,
}

#[derive(Debug)]
pub enum SegmentReadError {
    IndexFormatTooOld,
    IndexFormatTooNew,
    CorruptedIndex,
}

impl SegmentInfos {
    pub const MIN_VERSION: u32 = 9;
    pub const CURRENT_VERSION: u32 = 10;

    // TODO: Checksum for segment corruption
    pub fn read_commit(index_path: &Path, generation: u32) -> Result<Self, SegmentReadError> {
        let directory_reader = DirectoryReader { path: index_path };
        let mut index_input = directory_reader.open(&get_segment_file_name(generation));

        let magic = index_input.read_magic();

        if magic != Codec::MAGIC {
            return Err(SegmentReadError::IndexFormatTooOld);
        }

        let _version = Self::check_header(&mut index_input)?;

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

        return Ok(SegmentInfos {
            index_created_version,
            lucene_version,
            generation,
            id,
        });
    }

    /**
     * Checks header and returns version if all good
     */
    pub fn check_header(index_input: &mut IndexInput) -> Result<u32, SegmentReadError> {
        let codec = index_input.read_variable_string();

        if codec != Codec::SEGMENTS_CODEC {
            return Err(SegmentReadError::CorruptedIndex);
        }

        let version = index_input.read_version();

        match version {
            v if v < Self::MIN_VERSION => Err(SegmentReadError::IndexFormatTooOld),
            v if v > Self::CURRENT_VERSION => Err(SegmentReadError::IndexFormatTooNew),
            v => Ok(v),
        }
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
}
