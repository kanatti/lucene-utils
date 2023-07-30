use std::{collections::{HashMap, HashSet}, string::FromUtf8Error};

use bytes::{Buf, Bytes};

pub const ID_LENGTH: usize = 16;

pub fn read_string_fixed(bytes: &mut Bytes, length: usize) -> Result<String, FromUtf8Error> {
    match length {
        0 => Ok(String::new()),
        _ => String::from_utf8(read_bytes(bytes, length)),
    }
}

pub fn read_bytes(bytes: &mut Bytes, length: usize) -> Vec<u8> {
    match length {
        0 => Vec::new(),
        _ => bytes.copy_to_bytes(length).to_vec(),
    }
}

pub fn read_string(bytes: &mut Bytes) -> Result<String, FromUtf8Error> {
    let length = read_vint(bytes) as usize;

    read_string_fixed(bytes, length)
}

pub fn read_vint(bytes: &mut Bytes) -> u32 {
    let mut value: u32 = 0;
    let mut shift = 0;

    loop {
        let b = bytes.get_u8();
        value |= ((b & 0x7F) as u32) << shift;
        shift += 7;

        if b & 0x80 == 0 {
            break;
        }
    }

    value
}

pub fn read_id(bytes: &mut Bytes) -> Vec<u8> {
    read_bytes(bytes, ID_LENGTH)
}

/// Reads a HashMap<String, String> from a Bytes buffer.
pub fn read_map(bytes: &mut Bytes) -> HashMap<String, String> {
    let count = read_vint(bytes) as usize;
    let mut map = HashMap::with_capacity(count);

    for _ in 0..count {
        let key = read_string(bytes).unwrap();
        let value = read_string(bytes).unwrap();

        map.insert(key, value);
    }

    map
}

pub fn read_set(bytes: &mut Bytes) -> HashSet<String> {
    let count = read_vint(bytes) as usize;
    let mut set = HashSet::with_capacity(count);

    for _ in 0..count {
        let value = read_string(bytes).unwrap();
        set.insert(value);
    }

    set
}

pub fn read_vec(bytes: &mut Bytes) -> Vec<String> {
    let count = read_vint(bytes) as usize;
    let mut vec = Vec::with_capacity(count);

    for _ in 0..count {
        let value = read_string(bytes).unwrap();
        vec.push(value);
    }

    vec
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vint_one_byte() {
        let mut bytes = Bytes::from(vec![0x02]);

        let result = read_vint(&mut bytes);

        assert_eq!(result, 2);
    }

    #[test]
    fn test_vint_two_bytes() {
        let mut bytes = Bytes::from(vec![0x81, 0x01]);
        let result = read_vint(&mut bytes);

        assert_eq!(result, 129)
    }

    #[test]
    fn test_vint_two_bytes_max() {
        let mut bytes = Bytes::from(vec![0xFF, 0x7F]);
        let result = read_vint(&mut bytes);

        assert_eq!(result, 16383)
    }

    #[test]
    fn test_vint_three_bytes() {
        let mut bytes = Bytes::from(vec![0x81, 0x80, 0x01]);
        let result = read_vint(&mut bytes);

        assert_eq!(result, 16385)
    }
}
