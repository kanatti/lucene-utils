use std::string::FromUtf8Error;

use bytes::{Buf, Bytes};

pub fn read_string(bytes: &mut Bytes, length: usize) -> Result<String, FromUtf8Error> {
    String::from_utf8(read_bytes(bytes, length))
}

pub fn read_bytes(bytes: &mut Bytes, length: usize) -> Vec<u8> {
    bytes.copy_to_bytes(length).to_vec()
}

pub fn read_variable_string(bytes: &mut Bytes) -> Result<String, FromUtf8Error> {
    let length = read_variable_int(bytes) as usize;

    read_string(bytes, length)
}

// Fix: Incomplete, add cases to handle variable between one and five bytes
pub fn read_variable_int(bytes: &mut Bytes) -> u32 {
    bytes.get_u8().into()
}
