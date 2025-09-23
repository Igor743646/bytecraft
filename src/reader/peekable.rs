//! Traits and implementations for peeking at structured binary data.
//!
//! The `peekable` module provides the [`Peekable`] trait system that enables
//! ByteCraft to inspect data structures from binary streams without consuming
//! them. This is useful for lookahead operations, validation, and conditional
//! parsing where you need to examine data before deciding how to process it.
//!
//! # Core Concepts
//!
//! ## Peekable Trait
//!
//! The [`Peekable`] trait is the counterpart to [`Readable`][super::Readable], allowing types
//! to be inspected from a [`PeekStream`] without advancing the reader's position.
//! This enables non-destructive examination of data.
//!
//! ## Difference from Readable
//!
//! While [`Readable`][super::Readable] consumes data and advances the stream position, [`Peekable`]
//! leaves the position unchanged:
//!
//! ```rust
//! use bytecraft::{
//!     reader::ByteReader,
//!     readable::Readable,
//!     peekable::Peekable
//! };
//!
//! let data = [0x01, 0x02, 0x03, 0x04];
//! let mut reader = ByteReader::new(&data[..]);
//!
//! // Peek at data without consuming it
//! let peeked: u16 = reader.peek().unwrap();
//! assert_eq!(reader.position(), 0); // Position unchanged
//!
//! // Read data and consume it
//! let read: u16 = reader.read().unwrap();
//! assert_eq!(reader.position(), 2); // Position advanced
//! ```
//!
//! # Built-in Implementations
//!
//! ByteCraft provides `Peekable` implementations for the same standard types
//! that implement `Readable`, except tupples and arrays.
//!
//! ## Primitive Types
//!
//! All numeric primitives support endianness-aware peeking:
//!
//! ```rust
//! use bytecraft::{reader::ByteReader, common::Endian};
//!
//! let data = [0x12, 0x34, 0x56, 0x78];
//! let reader = ByteReader::with_endian(&data[..], Endian::Big);
//!
//! let value: u32 = reader.peek().unwrap();
//! assert_eq!(value, 0x12345678);
//! assert_eq!(reader.position(), 0); // Position unchanged
//! ```
//!
//! # Use Cases
//!
//! ## Protocol Detection
//!
//! Peek at magic numbers or version fields to determine how to parse data:
//!
//! ```rust
//! use bytecraft::{
//!     reader::ByteReader,
//!     peekable::Peekable,
//!     error::Result
//! };
//!
//! #[derive(Debug)]
//! enum FileFormat {
//!     Png,
//!     Jpeg,
//!     Unknown,
//! }
//!
//! impl FileFormat {
//!     fn detect(reader: &ByteReader<impl AsRef<[u8]>>) -> Result<Self> {
//!         let magic: u32 = reader.peek()?;
//!         match magic {
//!             0x89504E47 => Ok(FileFormat::Png),  // PNG magic
//!             0xFFD8FFE0 => Ok(FileFormat::Jpeg), // JPEG magic
//!             _ => Ok(FileFormat::Unknown),
//!         }
//!     }
//! }
//! ```
//!
//! ## Conditional Parsing
//!
//! Peek at flags or type indicators to choose parsing strategy:
//!
//! ```rust
//! use bytecraft::{
//!     reader::{ByteReader, ReadStream},
//!     peekable::Peekable,
//!     readable::Readable,
//!     error::{Error, Result}
//! };
//!
//! #[derive(Debug)]
//! enum Message {
//!     Text(String),
//!     Binary(Vec<u8>),
//! }
//!
//! impl Readable for Message {
//!     fn read<T: AsRef<[u8]>>(mut stream: ReadStream<T>) -> Result<Self> {
//!         let is_text: u8 = stream.peek()?; // Peek at type flag
//!         
//!         if is_text == 1 {
//!             stream.read_exact(1)?; // Consume the flag
//!             let len: u32 = stream.read()?;
//!             let data: &[u8] = stream.read_exact(len as usize)?;
//!             let text = str::from_utf8(data)
//!                 .map_err(|e| Error::NotValidUTF8(e))?;
//!             Ok(Message::Text(text.to_string()))
//!         } else {
//!             stream.read_exact(1)?; // Consume the flag
//!             let len: u32 = stream.read()?;
//!             let data: Vec<u8> = stream.read_exact(len as usize)?.into();
//!             Ok(Message::Binary(data))
//!         }
//!     }
//! }
//! ```
//!
//! # Custom Implementation Guide
//!
//! To make your type `Peekable`, implement the trait:
//!
//! ```rust
//! use bytecraft::{
//!     reader::{ByteReader, PeekStream},
//!     peekable::Peekable,
//!     error::Result
//! };
//!
//! #[derive(Debug, PartialEq)]
//! struct HeaderPreview {
//!     magic: u32,
//!     version: u16,
//! }
//!
//! impl Peekable for HeaderPreview {
//!     fn peek<T: AsRef<[u8]>>(s: PeekStream<T>) -> Result<Self> {
//!         let mut subreader = ByteReader::with_endian(s.peek_exact(6)?, s.get_endian());
//!         let magic: u32 = subreader.read()?;
//!         let version: u16 = subreader.read()?;  
//!
//!         Ok(HeaderPreview {
//!             magic,
//!             version,
//!         })
//!     }
//! }
//!
//! // Usage
//! fn main() -> Result<()> {
//!     let data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
//!     let reader = ByteReader::new(&data[..]);
//!     let preview: HeaderPreview = reader.peek()?;
//!     assert_eq!(reader.position(), 0); // Position unchanged
//!     assert_eq!(preview, HeaderPreview {magic: 0x03020100, version: 0x0504 });
//!     Ok(())
//! }
//! ```
//!
//! # Relationship to Readable
//!
//! Many types implement both traits, allowing flexible usage:
//!
//! ```rust
//! use bytecraft::{
//!     reader::ByteReader,
//!     readable::Readable,
//!     peekable::Peekable
//! };
//!
//! let data = [0x01, 0x02, 0x03, 0x04];
//! let mut reader = ByteReader::new(&data[..]);
//!
//! // Preview the data
//! let preview: u32 = reader.peek().unwrap();
//! println!("Will read value: {}", preview);
//!
//! // Then consume it
//! let actual: u32 = reader.read().unwrap();
//! assert_eq!(preview, actual);
//! ```

use crate::common::Endian;
use crate::error::{Error, Result};
use crate::reader::PeekStream;

/// A trait for types that can be peeked at from a binary stream.
///
/// The `Peekable` trait defines how a type should be inspected from a
/// sequence of bytes without consuming the data. Unlike [`Readable`][super::Readable], peeking
/// does not advance the stream position, allowing the same data to be read
/// again later.
///
/// # Safety
///
/// Implementations must ensure that:
/// - All peeks are properly bounds-checked
/// - Memory safety is maintained
/// - The stream position remains unchanged
/// - Errors are propagated appropriately
/// - Calling peek does not cause recursion
///
/// # Composition
///
/// `Peekable` implementations can not compose other `Peekable` types, because
/// the stream position does not change during a call.
/// Use another ByteReader to peek complex types.
///
/// # Examples
///
/// ## Simple Implementation
///
/// ```rust
/// use bytecraft::{
///     reader::{ByteReader, PeekStream},
///     peekable::Peekable,
///     error::Result
/// };
///
/// struct PreviewStruct {
///     id: u32,
///     flag: bool,
/// }
///
/// impl Peekable for PreviewStruct {
///     fn peek<T: AsRef<[u8]>>(s: PeekStream<T>) -> Result<Self> {
///         let mut subreader = ByteReader::with_endian(s.peek_exact(5)?, s.get_endian());
///         let (id, flag) = subreader.read::<(u32, bool)>()?;
///
///         Ok(PreviewStruct {
///             id,
///             flag,
///         })
///     }
/// }
/// ```
///
/// ## Complex Implementation with Custom error handling
///
/// ```rust
/// use bytecraft::{
///     reader::PeekStream,
///     peekable::Peekable,
///     error::{Result, Error}
/// };
///
/// #[derive(Debug)]
/// struct ValidatedPreview {
///     value: u32,
/// }
///
/// impl Peekable for ValidatedPreview {
///     fn peek<T: AsRef<[u8]>>(stream: PeekStream<T>) -> Result<Self> {
///         let value: u32 = stream.peek()?;
///         
///         // Custom validation without consuming data
///         if value > 1000 {
///             return Err(Error::NotValid);
///         }
///         
///         Ok(ValidatedPreview { value })
///     }
/// }
/// ```
///
/// # See Also
///
/// - [`crate::reader::ByteReader::peek()`] - The primary method for peeking at `Peekable` types
/// - [`crate::reader::PeekStream`] - The stream type provided to implementations
/// - [`Readable`][super::Readable] - The consuming counterpart to this trait
pub trait Peekable: Sized {
    /// Peeks at a value of this type from the provided stream.
    ///
    /// This method is called by [ByteReader::peek()][super::ByteReader::peek] to inspect values without
    /// consuming them. The implementation should examine exactly the number of
    /// bytes required for this type but must not advance the stream position.
    ///
    /// # Parameters
    ///
    /// - `stream`: A [`PeekStream`] providing access to the binary data
    ///
    /// # Returns
    ///
    /// - `Ok(value)` containing the peeked value
    /// - An error if peeking fails or data is invalid
    ///
    /// # Implementation Notes
    ///
    /// - Use `stream.peek::<T>()?` to peek at other `Peekable` types
    /// - Use `stream.peek_exact(size)?` for raw byte access
    /// - Consider endianness when peeking at multi-byte values
    /// - Handle errors appropriately with meaningful error types
    /// - **Never** advance the stream position
    fn peek<T: AsRef<[u8]>>(stream: PeekStream<T>) -> Result<Self>;
}

macro_rules! impl_number {
    ($Type:ty) => {
        impl Peekable for $Type {
            /// Peeks a numeric value with proper endianness handling.
            ///
            /// # Process
            /// 1. Peeks exactly `size_of::<$Type>()` bytes from the stream
            /// 2. Converts the value from bytes using appropriate endian conversion
            ///
            /// # Endianness Handling
            ///
            /// The conversion method depends on the stream's endianness setting:
            /// - [`Endian::Little`]: Uses `from_le_bytes()`
            /// - [`Endian::Big`]: Uses `from_be_bytes()`
            /// - [`Endian::Native`]: Uses `from_ne_bytes()`
            fn peek<T: AsRef<[u8]>>(s: PeekStream<T>) -> Result<Self> {
                let data: &[u8] = s.peek_exact(size_of::<$Type>())?;
                let data: [u8; size_of::<$Type>()] = data.try_into().unwrap();

                match s.get_endian() {
                    Endian::Little => Ok(<$Type>::from_le_bytes(data)),
                    Endian::Big => Ok(<$Type>::from_be_bytes(data)),
                    Endian::Native => Ok(<$Type>::from_ne_bytes(data)),
                }
            }
        }
    };
}

impl_number!(u8);
impl_number!(u16);
impl_number!(u32);
impl_number!(u64);
impl_number!(u128);
impl_number!(usize);
impl_number!(i8);
impl_number!(i16);
impl_number!(i32);
impl_number!(i64);
impl_number!(i128);
impl_number!(isize);
impl_number!(f32);
impl_number!(f64);

impl Peekable for bool {
    /// Peeks a byte ([u8]) and converts it to bool.
    /// - 0 - false
    /// - 1 - true
    ///
    /// Returns error [Error::NotValid] if value is not 0 or 1.
    fn peek<T: AsRef<[u8]>>(s: PeekStream<T>) -> Result<Self> {
        match s.peek::<u8>() {
            Ok(0) => Ok(false),
            Ok(1) => Ok(true),
            Ok(_) => Err(Error::NotValid),
            Err(err) => Err(err),
        }
    }
}
