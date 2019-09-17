//! All binary files generated by measureme have a simple file header that
//! consists of a 4 byte file magic string and a 4 byte little-endian version
//! number.

use crate::serialization::SerializationSink;
use byteorder::{ByteOrder, LittleEndian};
use std::error::Error;

pub const CURRENT_FILE_FORMAT_VERSION: u32 = 0;
pub const FILE_MAGIC_EVENT_STREAM: &[u8; 4] = b"MMES";
pub const FILE_MAGIC_STRINGTABLE_DATA: &[u8; 4] = b"MMSD";
pub const FILE_MAGIC_STRINGTABLE_INDEX: &[u8; 4] = b"MMSI";

/// The size of the file header in bytes. Note that functions in this module
/// rely on this size to be `8`.
pub const FILE_HEADER_SIZE: usize = 8;

pub fn write_file_header<S: SerializationSink>(s: &S, file_magic: &[u8; 4]) {
    // The implementation here relies on FILE_HEADER_SIZE to have the value 8.
    // Let's make sure this assumption cannot be violated without being noticed.
    assert_eq!(FILE_HEADER_SIZE, 8);

    s.write_atomic(FILE_HEADER_SIZE, |bytes| {
        bytes[0..4].copy_from_slice(file_magic);
        LittleEndian::write_u32(&mut bytes[4..8], CURRENT_FILE_FORMAT_VERSION);
    });
}

pub fn read_file_header(bytes: &[u8], expected_magic: &[u8; 4]) -> Result<u32, Box<dyn Error>> {
    // The implementation here relies on FILE_HEADER_SIZE to have the value 8.
    // Let's make sure this assumption cannot be violated without being noticed.
    assert_eq!(FILE_HEADER_SIZE, 8);

    let actual_magic = &bytes[0..4];

    if actual_magic != expected_magic {
        // FIXME: The error message should mention the file path in order to be
        //        more useful.
        let msg = format!(
            "Unexpected file magic `{:?}`. Expected `{:?}`",
            actual_magic, expected_magic,
        );

        return Err(From::from(msg));
    }

    Ok(LittleEndian::read_u32(&bytes[4..8]))
}

pub fn strip_file_header(data: &[u8]) -> &[u8] {
    &data[FILE_HEADER_SIZE..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::test::TestSink;

    #[test]
    fn roundtrip() {
        let data_sink = TestSink::new();

        write_file_header(&data_sink, FILE_MAGIC_EVENT_STREAM);

        let data = data_sink.into_bytes();

        assert_eq!(
            read_file_header(&data, FILE_MAGIC_EVENT_STREAM).unwrap(),
            CURRENT_FILE_FORMAT_VERSION
        );
    }

    #[test]
    fn invalid_magic() {
        let data_sink = TestSink::new();
        write_file_header(&data_sink, FILE_MAGIC_STRINGTABLE_DATA);
        let mut data = data_sink.into_bytes();

        // Invalidate the filemagic
        data[2] = 0;
        assert!(read_file_header(&data, FILE_MAGIC_STRINGTABLE_DATA).is_err());
    }

    #[test]
    fn other_version() {
        let data_sink = TestSink::new();

        write_file_header(&data_sink, FILE_MAGIC_STRINGTABLE_INDEX);

        let mut data = data_sink.into_bytes();

        // Change version
        data[4] = 0xFF;
        data[5] = 0xFF;
        data[6] = 0xFF;
        data[7] = 0xFF;
        assert_eq!(
            read_file_header(&data, FILE_MAGIC_STRINGTABLE_INDEX).unwrap(),
            0xFFFF_FFFF
        );
    }
}
