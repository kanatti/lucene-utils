use std::path::Path;

use bytes::{Buf, Bytes};

pub const ID_LENGTH: usize = 16;

pub struct IndexInput {
    bytes: Bytes,
}

impl IndexInput {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Bytes::from(bytes),
        }
    }

    pub fn read_magic(&mut self) -> u32 {
        self.bytes.get_u32()
    }

    pub fn read_variable_string(&mut self) -> String {
        let length = self.read_variable_int() as usize;

        return self.read_string(length);
    }

    pub fn read_string(&mut self, length: usize) -> String {
        return String::from_utf8(self.read_bytes(length)).unwrap();
    }

    // Fix: Incomplete, add cases to handle variable between one and five bytes
    pub fn read_variable_int(&mut self) -> u32 {
        self.bytes.get_u8().into()
    }

    pub fn read_version(&mut self) -> u32 {
        self.bytes.get_u32()
    }

    pub fn read_id(&mut self) -> Vec<u8> {
        self.read_bytes(ID_LENGTH)
    }

    pub fn read_byte(&mut self) -> u8 {
        self.bytes.get_u8()
    }

    pub fn read_bytes(&mut self, length: usize) -> Vec<u8> {
        self.bytes.copy_to_bytes(length).to_vec()
    }
}

pub struct DirectoryReader<'a> {
    pub path: &'a Path,
}

impl<'a> DirectoryReader<'a> {
    pub fn open(&self, segment_file_name: &str) -> IndexInput {
        println!("Opening segment - {}", segment_file_name);

        let file_path = self.path.join(segment_file_name);

        IndexInput::new(std::fs::read(file_path).unwrap())
    }
}
