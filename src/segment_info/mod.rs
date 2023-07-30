pub mod format;

#[derive(Debug)]
pub struct SegmentInfo {
    pub name: String,
    pub id: Vec<u8>,
}