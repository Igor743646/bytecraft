use bytecraft::error::Result;
use bytecraft::readable::Readable;
use bytecraft::reader::{ByteReader, ReadStream};
use bytecraft::writer::writable::Writable;
use bytecraft::writer::{ByteWriter, WriteStream};
use std::ffi::CString;

#[test]
fn test_vec_simple_types() -> Result<()> {
    let original_vec: Vec<u32> = vec![1, 2, 3, 4, 5];

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_vec)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_vec: Vec<u32> = reader.read()?;

    assert_eq!(original_vec, read_vec);
    Ok(())
}

#[test]
fn test_vec_complex_types() -> Result<()> {
    #[derive(Debug, PartialEq)]
    struct Point {
        x: f32,
        y: f32,
    }

    impl Readable for Point {
        fn read<T: AsRef<[u8]>>(mut s: ReadStream<T>) -> Result<Self> {
            Ok(Point {
                x: s.read()?,
                y: s.read()?,
            })
        }
    }

    impl Writable for Point {
        fn write<T>(mut s: WriteStream<T>, val: &Self) -> Result<()>
        where
            T: AsRef<[u8]> + AsMut<[u8]>,
        {
            s.write(&val.x)?;
            s.write(&val.y)?;
            Ok(())
        }
    }

    let original_vec: Vec<Point> = vec![
        Point { x: 1.0, y: 2.0 },
        Point { x: 3.0, y: 4.0 },
        Point { x: 5.0, y: 6.0 },
    ];

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_vec)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_vec: Vec<Point> = reader.read()?;

    assert_eq!(original_vec, read_vec);
    Ok(())
}

#[test]
fn test_string_roundtrip() -> Result<()> {
    let original_string: String = "Hello, 世界! 🦀".to_string();

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_string)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_string: String = reader.read()?;

    assert_eq!(original_string, read_string);
    Ok(())
}

#[test]
fn test_string_empty() -> Result<()> {
    let original_string: String = "".to_string();

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_string)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_string: String = reader.read()?;

    assert_eq!(original_string, read_string);
    Ok(())
}

#[test]
fn test_cstring_roundtrip() -> Result<()> {
    let original_cstring: CString = CString::new("Hello, World!").unwrap();

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_cstring)?;
    writer.write(&original_cstring)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_cstring1: CString = reader.read()?;
    let read_cstring2: CString = reader.read()?;

    assert_eq!(original_cstring, read_cstring1);
    assert_eq!(original_cstring, read_cstring2);
    Ok(())
}

#[test]
fn test_mixed_collections() -> Result<()> {
    let original_string: String = "Test data".to_string();
    let original_vec: Vec<i32> = vec![10, 20, 30];
    let original_cstring: CString = CString::new("C string").unwrap();

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);

    writer.write(&original_string)?;
    writer.write(&original_vec)?;
    writer.write(&original_cstring)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_string: String = reader.read()?;
    let read_vec: Vec<i32> = reader.read()?;
    let read_cstring: CString = reader.read()?;

    assert_eq!(original_string, read_string);
    assert_eq!(original_vec, read_vec);
    assert_eq!(original_cstring, read_cstring);
    Ok(())
}

#[test]
fn test_vec_size_limit() -> Result<()> {
    let original_vec: Vec<u8> = (0..1000usize).map(|x| (x % 256) as u8).collect();

    let mut buffer: Vec<u8> = vec![0u8; 2048];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_vec)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_vec: Vec<u8> = reader.read()?;

    assert_eq!(original_vec, read_vec);
    Ok(())
}

#[test]
fn test_string_unicode() -> Result<()> {
    let original_string: String = "🦀🚀⭐🌟🎉".to_string();

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_string)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_string: String = reader.read()?;

    assert_eq!(original_string, read_string);
    Ok(())
}

#[test]
fn test_empty_vec() -> Result<()> {
    let original_vec: Vec<u32> = vec![];

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
    writer.write(&original_vec)?;

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
    let read_vec: Vec<u32> = reader.read()?;

    assert_eq!(original_vec, read_vec);
    Ok(())
}

#[test]
fn test_collection_position_tracking() -> Result<()> {
    let string1: String = "First".to_string();
    let vec: Vec<u16> = vec![1, 2, 3];
    let string2: String = "Second".to_string();

    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);

    let start_pos: usize = writer.position();
    writer.write(&string1)?;
    let string1_end: usize = writer.position();
    writer.write(&vec)?;
    let vec_end: usize = writer.position();
    writer.write(&string2)?;
    let final_pos: usize = writer.position();

    assert!(string1_end > start_pos);
    assert!(vec_end > string1_end);
    assert!(final_pos > vec_end);

    let mut reader: ByteReader<_> = ByteReader::new(&buffer[..final_pos]);
    let read_string1: String = reader.read()?;
    let read_vec: Vec<u16> = reader.read()?;
    let read_string2: String = reader.read()?;

    assert_eq!(string1, read_string1);
    assert_eq!(vec, read_vec);
    assert_eq!(string2, read_string2);
    Ok(())
}
