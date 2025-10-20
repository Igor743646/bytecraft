//! Error and Result types for the ByteCraft library.
//!
//! This module provides comprehensive error handling for all operations
//! performed by ByteCraft. All functions in the library return a `Result<T>`
//! type that can contain any of the errors defined in the `Error` enum.
//!
//! # Examples
//!
//! ```
//! use bytecraft::error::{Error, Result};
//! use bytecraft::reader::ByteReader;
//!
//! fn read_u32_from_data(data: &[u8]) -> Result<u32> {
//!     let mut reader = ByteReader::new(data);
//!     reader.read::<u32>() // Returns Result<u32, Error>
//! }
//!
//! let short_data = [0x01, 0x02]; // Only 2 bytes
//! match read_u32_from_data(&short_data) {
//!     Ok(value) => println!("Read value: {}", value),
//!     Err(Error::InsufficientData { requested, available }) => {
//!         println!("Need {} bytes, but only {} available", requested, available);
//!     }
//!     Err(e) => println!("Other error: {}", e),
//! }
//! ```

use core::str::Utf8Error;

/// The error type for ByteCraft operations.
///
/// This enum represents all possible errors that can occur when working
/// with binary data using ByteCraft. Each variant provides detailed
/// information about what went wrong to help with debugging and error handling.
///
/// # Error Handling Patterns
///
/// ```
/// use bytecraft::error::{Error, Result};
/// use bytecraft::reader::ByteReader;
///
/// fn process_data(data: &[u8]) -> Result<String> {
///     let mut reader = ByteReader::new(data);
///     
///     // Handle specific errors
///     let value = match reader.read::<u32>() {
///         Ok(v) => v,
///         Err(Error::InsufficientData { requested, available }) => {
///             return Err(Error::Custom(
///                 format!("Not enough data: need {}, have {}", requested, available).into()
///             ));
///         }
///         Err(e) => return Err(e),
///     };
///     
///     Ok(value.to_string())
/// }
/// ```
#[derive(Debug)]
pub enum Error {
    /// Insufficient data available for the requested operation.
    ///
    /// This error occurs when trying to read more bytes than are available
    /// in the remaining data stream. It provides exact information about
    /// how much data was requested versus how much is actually available.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::error::Error;
    /// use bytecraft::reader::ByteReader;
    ///
    /// let data = [0x01, 0x02]; // Only 2 bytes
    /// let mut reader = ByteReader::new(&data[..]);
    ///
    /// // Trying to read u32 (4 bytes) from 2 bytes of data
    /// match reader.read::<u32>() {
    ///     Err(Error::InsufficientData { requested, available }) => {
    ///         assert_eq!(requested, 4);
    ///         assert_eq!(available, 2);
    ///     }
    ///     _ => panic!("Expected InsufficientData error"),
    /// }
    /// ```
    ///
    /// # When This Error Occurs
    ///
    /// - Reading primitive types when not enough bytes remain
    /// - Reading strings or arrays of specific sizes
    /// - Any operation that requires more data than available
    InsufficientData {
        /// The number of bytes requested for the operation
        requested: usize,
        /// The number of bytes actually available in the stream
        available: usize,
    },

    /// Attempted to seek or set position outside the valid data boundaries.
    ///
    /// This error occurs when trying to set the cursor position to an invalid
    /// location, either beyond the end of the data or to a negative position.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::error::Error;
    /// use bytecraft::reader::ByteReader;
    /// use bytecraft::common::SeekFrom;
    ///
    /// let data = [0x01, 0x02, 0x03, 0x04];
    /// let mut reader = ByteReader::new(&data[..]);
    ///
    /// // Trying to seek beyond the end of data
    /// match reader.seek(SeekFrom::Start(100)) {
    ///     Err(Error::OutOfBounds { pos, requested, len }) => {
    ///         assert_eq!(pos, 0); // current position
    ///         assert_eq!(requested, 100); // requested position
    ///         assert_eq!(len, 4); // actual data length
    ///     }
    ///     _ => panic!("Expected OutOfBounds error"),
    /// }
    /// ```
    ///
    /// # When This Error Occurs
    ///
    /// - Setting position beyond data length using `set_position()`
    /// - Seeking to invalid positions using `seek()`
    /// - Skipping more bytes than available using `skip()`
    OutOfBounds {
        /// The current position in the data stream
        pos: usize,
        /// The requested position or shift that caused the error
        requested: usize,
        /// The actual length of the data
        len: usize,
    },

    /// Data is not valid for the requested operation.
    ///
    /// This is a generic error for cases where the data format is invalid
    /// or doesn't match expected patterns. It's typically used when more
    /// specific error types don't apply.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::error::Error;
    /// use bytecraft::reader::ByteReader;
    ///
    /// // This might occur when parsing custom formats with invalid flags
    /// // or when data doesn't match expected scheme, pattern or magic numbers
    /// ```
    ///
    /// # When This Error Occurs
    ///
    /// - Invalid magic numbers or file signatures
    /// - Unexpected flag values in binary protocols
    /// - Data that doesn't conform to expected format specifications
    NotValid,

    /// Data contains non-ASCII characters when ASCII was expected.
    ///
    /// This error occurs specifically when trying to read ASCII strings
    /// and the data contains bytes that are not valid ASCII characters
    /// (values outside the 0-127 range).
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::error::Error;
    /// use bytecraft::reader::ByteReader;
    ///
    /// // Data containing non-ASCII bytes
    /// let data: &[u8] = "📚".as_bytes(); // Valid UTF-8 but non-ASCII // "\xF0\x9F\x93\x9A"
    /// let mut reader: ByteReader = ByteReader::new(data);
    ///
    /// match reader.read_ascii(4) {
    ///     Err(Error::NotValidAscii) => {
    ///         // Data contains non-ASCII bytes
    ///     }
    ///     _ => panic!("Expected NotValidAscii error"),
    /// }
    /// ```
    ///
    /// # When This Error Occurs
    ///
    /// - Using `read_ascii()` on data containing non-ASCII bytes
    /// - Parsing ASCII-only protocols with invalid character data
    NotValidAscii,

    /// UTF-8 string data is malformed or invalid.
    ///
    /// This error wraps the underlying `Utf8Error` from Rust's standard library
    /// and occurs when trying to convert byte sequences to UTF-8 strings that
    /// contain invalid UTF-8 sequences.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::error::Error;
    /// use bytecraft::reader::ByteReader;
    /// use core::str;
    ///
    /// // Invalid UTF-8 sequence
    /// let data = [0xC0, 0x80]; // Invalid UTF-8
    /// let mut reader = ByteReader::new(&data[..]);
    ///
    /// match reader.read_bytes(2).and_then(|bytes| Ok(str::from_utf8(bytes))) {
    ///     Err(_) => {
    ///         // Would result in NotValidUTF8 error if used in string reading methods
    ///     }
    ///     _ => {}
    /// }
    /// ```
    ///
    /// # When This Error Occurs
    ///
    /// - Reading UTF-8 strings from malformed byte sequences
    /// - Parsing text data that contains invalid UTF-8
    /// - Converting binary data to strings without proper validation
    NotValidUTF8(Utf8Error),

    /// A custom error from user-defined types or operations.
    ///
    /// This variant allows `Readable` and `Peekable` implementations to
    /// return their own custom errors while maintaining compatibility
    /// with the ByteCraft error system. It can contain any error type
    /// that implements `std::error::Error + Send + Sync`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::error::Error;
    /// use bytecraft::error::Result;
    /// use bytecraft::readable::Readable;
    /// use bytecraft::reader::ReadStream;
    /// use bytecraft::common::SeekFrom;
    /// use std::error::Error as StdError;
    ///
    /// #[derive(Debug)]
    /// struct CustomParseError(String);
    ///
    /// impl std::fmt::Display for CustomParseError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "Custom parse error: {}", self.0)
    ///     }
    /// }
    ///
    /// impl StdError for CustomParseError {}
    ///
    /// struct MyCustomType;
    ///
    /// impl<'a> Readable<'a> for MyCustomType {
    ///     fn read<'r>(mut s: ReadStream<'a, 'r>) -> Result<Self> {
    ///         // Some custom validation that fails
    ///         Err(Error::Custom(Box::new(CustomParseError("Invalid format".to_string()))))
    ///     }
    /// }
    /// ```
    ///
    /// # When This Error Occurs
    ///
    /// - User-defined `Readable` implementations returning custom errors
    /// - Domain-specific validation failures in complex data structures
    /// - Wrapping external library errors in ByteCraft's error system
    Custom(Box<dyn std::error::Error + Send + Sync>),
}

/// A specialized `Result` type for ByteCraft operations.
///
/// This type is used throughout the ByteCraft library for functions that
/// can fail. It's equivalent to `std::result::Result<T, Error>` but
/// provides a convenient shorthand.
///
/// # Examples
///
/// ```
/// use bytecraft::error::Result;
/// use bytecraft::reader::ByteReader;
///
/// fn read_header(data: &[u8]) -> Result<u32> {
///     let mut reader = ByteReader::new(data);
///     reader.read::<u32>() // Returns Result<u32, Error>
/// }
///
/// // Usage with pattern matching
/// match read_header(&[0x01, 0x02, 0x03, 0x04]) {
///     Ok(value) => println!("Header value: {}", value),
///     Err(e) => eprintln!("Failed to read header: {}", e),
/// }
/// ```
///
/// # Converting from std::result::Result
///
/// ```
/// use bytecraft::error::{Result, Error};
/// use std::io;
///
/// fn convert_io_result(result: io::Result<u32>) -> Result<u32> {
///     result.map_err(|e| Error::Custom(Box::new(e)))
/// }
/// ```
pub type Result<T> = core::result::Result<T, Error>;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InsufficientData {
                requested,
                available,
            } => {
                write!(
                    f,
                    "Insufficient data: requested {} bytes, but only {} available",
                    requested, available
                )
            }
            Error::OutOfBounds {
                pos,
                requested,
                len,
            } => {
                write!(
                    f,
                    "Position out of bounds: current={}, requested={}, length={}",
                    pos, requested, len
                )
            }
            Error::NotValid => {
                write!(f, "Data is not valid for the requested operation")
            }
            Error::NotValidAscii => {
                write!(f, "Data contains non-ASCII characters")
            }
            Error::NotValidUTF8(err) => {
                write!(f, "Invalid UTF-8 sequence: {}", err)
            }
            Error::Custom(err) => {
                write!(f, "Custom error: {}", err)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InsufficientData { .. } => None,
            Error::OutOfBounds { .. } => None,
            Error::NotValid => None,
            Error::NotValidAscii => None,
            Error::NotValidUTF8(err) => Some(err),
            Error::Custom(error) => error.source(),
        }
    }
}
