use std::collections::{HashMap, HashSet};

pub mod format;

#[derive(Debug)]
pub struct SegmentInfo {
    pub name: String,
    pub id: Vec<u8>,
    pub version: (u32, u32, u32),
    pub min_version: Option<(u32, u32, u32)>,
    pub doc_count: u32,
    pub is_compound_file: bool,
    pub diagnostics: HashMap<String, String>,
    pub files: HashSet<String>,
    pub attributes: HashMap<String, String>,
    pub sort_fields: Vec<String>,
}

#[derive(Debug)]
pub struct SegmentCommitInfo {
    pub info: SegmentInfo,
    pub del_count: u32,
    pub soft_del_count: u32,
    pub del_gen: i64,
    pub field_infos_gen: i64,
    pub dv_gen: i64,
    pub sci_id: Option<Vec<u8>>,
    pub field_info_files: HashSet<String>,
    pub dv_update_files: HashMap<u32, HashSet<String>>,
}
