use bytecraft::common::SeekFrom;
use bytecraft::{error::*, reader::ByteReader};

#[test]
fn test_insufficient_data_error() -> Result<()> {
    let error: Error = Error::InsufficientData {
        requested: 4,
        available: 2,
    };

    assert!(matches!(
        error,
        Error::InsufficientData {
            requested: 4,
            available: 2
        }
    ));

    assert_eq!(
        error.to_string(),
        "Insufficient data: requested 4 bytes, but only 2 available"
    );

    Ok(())
}

#[test]
fn test_out_of_bounds_error() -> Result<()> {
    let error: Error = Error::OutOfBounds {
        pos: 0,
        requested: 100,
        len: 4,
    };

    assert!(matches!(
        error,
        Error::OutOfBounds {
            pos: 0,
            requested: 100,
            len: 4
        }
    ));

    assert_eq!(
        error.to_string(),
        "Position out of bounds: current=0, requested=100, length=4"
    );

    Ok(())
}

#[test]
fn test_not_valid_error() -> Result<()> {
    let error: Error = Error::NotValid;
    assert_eq!(
        error.to_string(),
        "Data is not valid for the requested operation"
    );
    Ok(())
}

#[test]
fn test_not_valid_ascii_error() -> Result<()> {
    let error: Error = Error::NotValidAscii;
    assert_eq!(error.to_string(), "Data contains non-ASCII characters");
    Ok(())
}

#[test]
fn test_not_valid_ascii_error_doc() -> Result<()> {
    let data: &[u8] = "📚".as_bytes(); // Invalid UTF-8 and non-ASCII // "\xF0\x9F\x93\x9A"
    let mut reader: ByteReader = ByteReader::new(data);

    match reader.read_ascii(4) {
        Err(Error::NotValidAscii) => {
            // Data contains non-ASCII bytes
            Ok(())
        }
        Err(Error::NotValidUTF8(err)) => {
            // Data contains non-UTF8 bytes
            Err(Error::NotValidUTF8(err))
        }
        _ => panic!("Expected NotValidAscii error"),
    }
}

#[test]
#[allow(invalid_from_utf8)]
fn test_not_valid_utf8_error() -> Result<()> {
    let utf8_error: std::str::Utf8Error = std::str::from_utf8(b"\xC0\x80").unwrap_err();
    let error: Error = Error::NotValidUTF8(utf8_error);
    assert!(error.to_string().contains("Invalid UTF-8 sequence"));
    Ok(())
}

#[test]
fn test_custom_error() -> Result<()> {
    #[derive(Debug)]
    struct CustomError(String);

    impl std::fmt::Display for CustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Custom: {}", self.0)
        }
    }

    impl std::error::Error for CustomError {}

    let custom_error: CustomError = CustomError("test".to_string());
    let error: Error = Error::Custom(Box::new(custom_error));
    assert_eq!(error.to_string(), "Custom error: Custom: test");
    Ok(())
}

#[test]
fn test_insufficient_data_example() -> Result<()> {
    let data: [u8; 2] = [0x01, 0x02]; // Only 2 bytes
    let mut reader: ByteReader = ByteReader::new(&data);

    // Trying to read u32 (4 bytes) from 2 bytes of data
    match reader.read::<u32>() {
        Err(Error::InsufficientData {
            requested,
            available,
        }) => {
            assert_eq!(requested, 4);
            assert_eq!(available, 2);
            Ok(())
        }
        _ => panic!("Expected InsufficientData error"),
    }
}

#[test]
fn test_out_of_bounds_example() -> Result<()> {
    let data: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let mut reader: ByteReader = ByteReader::new(&data);

    // Trying to seek beyond the end of data
    match reader.seek(SeekFrom::Start(100)) {
        Err(Error::OutOfBounds {
            pos,
            requested,
            len,
        }) => {
            assert_eq!(pos, 0);
            assert_eq!(requested, 100);
            assert_eq!(len, 4);
            Ok(())
        }
        _ => panic!("Expected OutOfBounds error"),
    }
}

#[test]
fn test_result_type_usage() -> Result<()> {
    fn read_header(data: &[u8]) -> Result<u32> {
        let mut reader: ByteReader = ByteReader::new(data);
        reader.read::<u32>()
    }

    let result: Result<u32> = read_header(&[0x01, 0x02, 0x03, 0x04]);
    assert!(result.is_ok());
    Ok(())
}

#[test]
#[allow(invalid_from_utf8)]
fn test_custom_error_source() -> Result<()> {
    use bytecraft::error::Error as BCError;
    use std::error::Error;

    let utf8_error: std::str::Utf8Error = std::str::from_utf8(b"\xC0\x80").unwrap_err();
    let error: BCError = BCError::NotValidUTF8(utf8_error);

    let source: Option<_> = error.source();
    assert!(source.is_some());
    assert!(source.unwrap().is::<std::str::Utf8Error>());
    Ok(())
}

#[test]
fn test_error_source_none() -> Result<()> {
    use bytecraft::error::Error as BCError;
    use std::error::Error;

    let error: BCError = BCError::NotValid;
    let source: Option<_> = error.source();
    assert!(source.is_none());
    Ok(())
}
