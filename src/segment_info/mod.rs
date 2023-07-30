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