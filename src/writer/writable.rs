//! Traits and implementations for writing structured binary data.
//!
//! The `writable` module provides the core trait system that enables ByteCraft
//! to serialize complex data structures to binary streams. It defines how types
//! should be converted to byte sequences and includes implementations for
//! common Rust types.
//!
//! # Core Concepts
//!
//! ## Writable Trait
//!
//! The [`Writable`] trait is the foundation of ByteCraft's serialization system.
//! Any type that implements `Writable` can be written to a [`WriteStream`] using
//! the [ByteWriter::write()](crate::writer::ByteWriter::write) method.
//!
//! ## Composition and Nesting
//!
//! `Writable` implementations can compose other `Writable` types, enabling
//! complex nested structures:
//!
//! ```rust
//! use bytecraft::{
//!     writer::{ByteWriter, WriteStream},
//!     writable::Writable,
//!     error::Result
//! };
//!
//! #[derive(Debug)]
//! struct Header {
//!     magic: u32,
//!     version: u16,
//! };
//!
//! impl Writable for Header {
//!     fn write<T>(mut stream: WriteStream<T>, val: &Self) -> Result<()>
//!     where
//!         T: AsRef<[u8]> + AsMut<[u8]>,
//!     {
//!         stream.write(&val.magic)?;    // Writes u32
//!         stream.write(&val.version)?;  // Writes u16
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct File {
//!     header: Header,      // Nested Writable type
//!     data: Vec<u8>,       // Another Writable type
//! };
//!
//! impl Writable for File {
//!     fn write<T>(mut stream: WriteStream<T>, val: &Self) -> Result<()>
//!     where
//!         T: AsRef<[u8]> + AsMut<[u8]>,
//!     {
//!         stream.write(&val.header)?;              // Composes Header
//!         stream.write(&(val.data.len() as u32))?; // Writes length
//!         stream.write(&val.data)?;                // Writes data
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Built-in Implementations
//!
//! ByteCraft provides `Writable` implementations for many standard types:
//!
//! ## Primitive Types
//!
//! All numeric primitives support endianness-aware writing:
//!
//! ```rust
//! use bytecraft::{writer::ByteWriter, common::Endian};
//!
//! let mut buffer = [0u8; 4];
//! let mut writer = ByteWriter::with_endian(&mut buffer[..], Endian::Big);
//!
//! let value: u32 = 0x12345678;
//! writer.write(&value).unwrap();
//!
//! // Buffer now contains [0x12, 0x34, 0x56, 0x78]
//! assert_eq!(&buffer, &[0x12, 0x34, 0x56, 0x78]);
//! ```
//!
//! ## Arrays
//!
//! Fixed-size arrays where elements implement `Writable`:
//!
//! ```rust
//! use bytecraft::writer::ByteWriter;
//!
//! let mut buffer = [0u8; 5];
//! let mut writer = ByteWriter::new(&mut buffer[..]);
//!
//! let array: [u8; 3] = [1, 2, 3];
//! writer.write(&array).unwrap();
//!
//! assert_eq!(&buffer[..3], &[1, 2, 3]);
//! ```
//!
//! ## Tuples
//!
//! Tuples up to 13 elements support structured writing:
//!
//! ```rust
//! use bytecraft::writer::ByteWriter;
//!
//! let mut buffer = [0u8; 4];
//! let mut writer = ByteWriter::new(&mut buffer[..]);
//!
//! let tuple: (u8, u16) = (1, 0x0203u16);
//! writer.write(&tuple).unwrap();
//!
//! // Assuming little endian: [1, 3, 2, 0]
//! ```
//!
//! ## Collections
//!
//! Standard collections with length-prefixing:
//!
//! ```rust
//! use bytecraft::writer::ByteWriter;
//!
//! let mut buffer = vec![0u8; 1024];
//! let mut writer = ByteWriter::new(&mut buffer[..]);
//!
//! // String with length prefix
//! let string = "Hello".to_string();
//! writer.write(&string).unwrap();
//!
//! // Vec with length prefix
//! let vec: Vec<u32> = vec![1, 2, 3];
//! writer.write(&vec).unwrap();
//! ```
//!
//! # Custom Implementation Guide
//!
//! To make your type `Writable`, implement the trait:
//!
//! ```rust
//! use bytecraft::{
//!     writer::{ByteWriter, WriteStream},
//!     writable::Writable,
//!     error::Result
//! };
//!
//! #[derive(Debug)]
//! struct Point {
//!     x: f32,
//!     y: f32,
//! }
//!
//! impl Writable for Point {
//!     fn write<T>(mut stream: WriteStream<T>, val: &Self) -> Result<()>
//!     where
//!         T: AsRef<[u8]> + AsMut<[u8]>,
//!     {
//!         stream.write(&val.x)?;
//!         stream.write(&val.y)?;
//!         Ok(())
//!     }
//! }
//!
//! // Usage
//! fn main() -> Result<()> {
//!     let point = Point { x: 1.0, y: 2.0 };
//!     let mut buffer = [0u8; 8];
//!     let mut writer = ByteWriter::new(&mut buffer[..]);
//!     writer.write(&point)?;
//!     Ok(())
//! }
//! ```
//!
//! # Safety Considerations
//!
//! Implementations must ensure that:
//! - All writes are properly bounds-checked
//! - Memory safety is maintained
//! - Errors are propagated appropriately
//! - The stream position is advanced correctly
//! - The implementation does not cause recursion
use std::ffi::CString;

use crate::common::Endian;
use crate::error::{Error, Result};
use crate::writer::WriteStream;

/// A trait for types that can be written to a binary stream.
///
/// The `Writable` trait defines how a type should be serialized to a
/// sequence of bytes. Implementations receive a [`WriteStream`] that provides
/// access to the underlying buffer and handles position management.
///
/// # Safety
///
/// Implementations must ensure that:
/// - All writes are properly bounds-checked
/// - Memory safety is maintained
/// - The stream position is advanced by exactly the number of bytes written
/// - Errors are propagated appropriately
/// - The implementation does not cause recursion
///
/// # Composition
///
/// `Writable` implementations can compose other `Writable` types, enabling
/// complex nested structures to be written seamlessly.
///
/// # Examples
///
/// ## Simple Implementation
///
/// ```rust
/// use bytecraft::{
///     writer::WriteStream,
///     writable::Writable,
///     error::Result
/// };
///
/// struct SimpleStruct {
///     id: u32,
///     flag: bool,
/// }
///
/// impl Writable for SimpleStruct {
///     fn write<T>(mut stream: WriteStream<T>, val: &Self) -> Result<()>
///     where
///         T: AsRef<[u8]> + AsMut<[u8]>,
///     {
///         stream.write(&val.id)?;
///         stream.write(&val.flag)?;
///         Ok(())
///     }
/// }
/// ```
///
/// ## Complex Implementation with Custom Logic
///
/// ```rust
/// use bytecraft::{
///     writer::WriteStream,
///     writable::Writable,
///     error::{Result, Error}
/// };
///
/// struct LengthPrefixedData {
///     data: Vec<u8>,
/// }
///
/// impl Writable for LengthPrefixedData {
///     fn write<T>(mut stream: WriteStream<T>, val: &Self) -> Result<()>
///     where
///         T: AsRef<[u8]> + AsMut<[u8]>,
///     {
///         if val.data.len() > u32::MAX as usize {
///             return Err(Error::NotValid);
///         }
///         
///         stream.write(&(val.data.len() as u32))?;
///         stream.write_exact(&val.data)?;
///         Ok(())
///     }
/// }
/// ```
///
/// # See Also
///
/// - [`crate::writer::ByteWriter::write()`] - The primary method for writing `Writable` types
/// - [`crate::writer::WriteStream`] - The stream type provided to implementations
pub trait Writable: Sized {
    /// Writes a value of this type to the provided stream.
    ///
    /// This method is called by [`ByteWriter::write()`](super::ByteWriter::write) to serialize values.
    /// The implementation should write exactly the number of bytes required
    /// for this type and advance the stream position accordingly.
    ///
    /// # Parameters
    ///
    /// - `stream`: A [`WriteStream`] providing access to the binary buffer
    /// - `val`: A reference to the value to be written
    ///
    /// # Type Parameters
    ///
    /// - `T`: The underlying buffer type that must support both reading and
    ///        writing operations through `AsRef<[u8]>` and `AsMut<[u8]>`
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the value was successfully written
    /// - An error if serialization fails or buffer space is insufficient
    ///
    /// # Implementation Notes
    ///
    /// - Use `stream.write(val)?` to write other `Writable` types
    /// - Use `stream.write_exact(bytes)?` for raw byte access
    /// - Consider endianness when writing multi-byte values
    /// - Handle errors appropriately with meaningful error types
    /// - Ensure the stream position is advanced correctly
    fn write<T>(stream: WriteStream<T>, val: &Self) -> Result<()>
    where
        T: AsRef<[u8]> + AsMut<[u8]>;
}

macro_rules! impl_number {
    ($Type:ty) => {
        impl Writable for $Type {
            /// Writes a numeric value with proper endianness handling.
            ///
            /// # Process
            /// 1. Converts the value to bytes using appropriate endian conversion
            /// 2. Writes exactly `size_of::<$Type>()` bytes to the stream
            /// 3. Advances the stream position by the number of bytes written
            ///
            /// # Endianness Handling
            ///
            /// The conversion method depends on the stream's endianness setting:
            /// - [`Endian::Little`]: Uses `to_le_bytes()`
            /// - [`Endian::Big`]: Uses `to_be_bytes()`
            /// - [`Endian::Native`]: Uses `to_ne_bytes()`
            fn write<T>(mut s: WriteStream<T>, val: &Self) -> Result<()>
            where
                T: AsRef<[u8]> + AsMut<[u8]>,
            {
                match s.get_endian() {
                    Endian::Little => s.write_exact(&val.to_le_bytes()),
                    Endian::Big => s.write_exact(&val.to_be_bytes()),
                    Endian::Native => s.write_exact(&val.to_ne_bytes()),
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

impl Writable for bool {
    /// Writes a boolean value as a single byte.
    ///
    /// # Process
    /// - `true` is written as the byte `0x01`
    /// - `false` is written as the byte `0x00`
    /// - Stream position is advanced by 1 byte
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 1];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// writer.write(&true).unwrap();
    /// assert_eq!(writer.as_slice()[0], 0x01);
    ///
    /// writer.reset();
    /// writer.write(&false).unwrap();
    /// assert_eq!(writer.as_slice()[0], 0x00);
    /// ```
    fn write<T>(mut s: WriteStream<T>, val: &bool) -> Result<()>
    where
        T: AsRef<[u8]> + AsMut<[u8]>,
    {
        match val {
            true => s.write_exact(&[0x01]),
            false => s.write_exact(&[0x00]),
        }
    }
}

impl Writable for char {
    /// Writes a character value as UTF-8 encoded bytes.
    ///
    /// # Process
    /// 1. Encodes the character as UTF-8 using up to 4 bytes
    /// 2. Writes the UTF-8 byte sequence to the stream
    /// 3. Advances the stream position by the number of bytes written
    ///
    /// # UTF-8 Encoding
    ///
    /// The number of bytes written depends on the character:
    /// - ASCII characters (0-127): 1 byte
    /// - Latin/Greek/Cyrillic: 2 bytes
    /// - CJK characters: 3 bytes
    /// - Emojis and rare characters: 4 bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// // ASCII character
    /// writer.write(&'A').unwrap();
    /// assert_eq!(&writer.as_slice()[..1], &[0x41]);
    ///
    /// // Multi-byte character
    /// writer.reset();
    /// writer.write(&'€').unwrap(); // Euro sign
    /// assert_eq!(&writer.as_slice()[..3], &[0xE2, 0x82, 0xAC]); // UTF-8 for €
    /// ```
    fn write<T>(mut s: WriteStream<T>, val: &char) -> Result<()>
    where
        T: AsRef<[u8]> + AsMut<[u8]>,
    {
        let mut buff: [u8; 4] = [0u8; 4];
        let bytes: &mut str = val.encode_utf8(&mut buff);
        s.write_exact(bytes.as_bytes())
    }
}

impl<T: Writable, const N: usize> Writable for [T; N] {
    /// Writes a fixed-size array by writing each element sequentially.
    ///
    /// # Process
    /// 1. Iterates through array elements in order (0 to N-1)
    /// 2. Writes each element using its `Writable` implementation
    /// 3. Advances the stream position by the each element
    ///
    /// # Element Order
    ///
    /// Elements are written in the same order as they appear in the array
    /// type definition, from index 0 to N-1.
    ///
    /// # Error Handling
    ///
    /// If any element fails to write, the error is immediately
    /// propagated and subsequent elements are not written.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 8];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// let array: [u16; 3] = [0x1234, 0x5678, 0x9ABC];
    /// writer.write(&array).unwrap();
    ///
    /// // Assuming little endian: [0x34, 0x12, 0x78, 0x56, 0xBC, 0x9A]
    /// ```
    fn write<U>(mut s: WriteStream<U>, val: &Self) -> Result<()>
    where
        U: AsRef<[u8]> + AsMut<[u8]>,
    {
        for item in val {
            s.write(item)?;
        }

        Ok(())
    }
}

macro_rules! impl_tupple {
    ($($Index:tt: $Types:tt)+) => {
        /// Writes a tuple by writing each field sequentially.
        ///
        /// # Process
        /// 1. Writes each type parameter in order using `stream.write()`
        /// 2. Advances the stream position by each element written
        ///
        /// # Element Order
        ///
        /// Elements are written in the same order as they appear in the tuple
        /// type definition, from left to right.
        ///
        /// # Error Propagation
        ///
        /// If any element fails to write, the error is immediately
        /// propagated and subsequent elements are not written.
        impl<$($Types : Writable),+> Writable for ($($Types ,)+) {
            fn write<S: AsRef<[u8]> + AsMut<[u8]>>(mut s: WriteStream<S>, val: &($($Types ,)+)) -> Result<()> {
                $(
                    s.write(&val.$Index)?;
                )+
                Ok(())
            }
        }
    };
}

// Tuple implementations from 1 to 13 elements
impl_tupple!(0: T0);
impl_tupple!(0: T0 1: T1);
impl_tupple!(0: T0 1: T1 2: T2);
impl_tupple!(0: T0 1: T1 2: T2 3: T3);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10 11: T11);
impl_tupple!(0: T0 1: T1 2: T2 3: T3 4: T4 5: T5 6: T6 7: T7 8: T8 9: T9 10: T10 11: T11 12: T12);

impl<T: Writable> Writable for Vec<T> {
    /// Writes a vector with length prefix and element data.
    ///
    /// # Process
    /// 1. Validates vector length fits in `u32`
    /// 2. Writes length as `u32`
    /// 3. Writes each element using its `Writable` implementation
    /// 4. Advances stream position by each element written
    ///
    /// # Error Handling
    ///
    /// - Returns [`Error::NotValid`] if vector length exceeds `u32::MAX`
    /// - Propagates errors from element writing immediately
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = vec![0u8; 1024];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// let vec: Vec<u8> = vec![1, 2, 3];
    /// writer.write(&vec).unwrap();
    ///
    /// // First 4 bytes are length (3), then elements
    /// assert_eq!(&buffer[..4], &[3, 0, 0, 0]); // Little endian 3
    /// assert_eq!(&buffer[4..7], &[1, 2, 3]);
    /// ```
    fn write<U>(mut s: WriteStream<U>, val: &Self) -> Result<()>
    where
        U: AsRef<[u8]> + AsMut<[u8]>,
    {
        if val.len() > u32::MAX as usize {
            return Err(Error::NotValid);
        }

        s.write(&(val.len() as u32))?;
        for val in val {
            s.write(val)?;
        }

        Ok(())
    }
}

impl Writable for String {
    /// Writes a string with length prefix and UTF-8 encoded data.
    ///
    /// # Process
    /// 1. Validates string length fits in `u32`
    /// 2. Writes length as `u32`
    /// 3. Writes UTF-8 encoded string bytes
    /// 4. Advances stream position by total bytes written
    ///
    /// # Error Handling
    ///
    /// - Returns [`Error::NotValid`] if string length exceeds `u32::MAX`
    /// - UTF-8 encoding is guaranteed by `String` type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = vec![0u8; 1024];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// let string = "🦀".to_string(); // Emoji is 4 UTF-8 bytes
    /// writer.write(&string).unwrap();
    ///
    /// // First 4 bytes: length (4), then UTF-8 bytes
    /// assert_eq!(&buffer[..4], &[4, 0, 0, 0]); // Little endian 4
    /// // Next 4 bytes: UTF-8 encoding of 🦀
    /// ```
    fn write<T>(mut s: WriteStream<T>, val: &Self) -> Result<()>
    where
        T: AsRef<[u8]> + AsMut<[u8]>,
    {
        if val.len() > u32::MAX as usize {
            return Err(Error::NotValid);
        }

        s.write(&(val.len() as u32))?;
        s.write_exact(val.as_bytes())
    }
}

impl Writable for CString {
    /// Writes a C string including its null terminator.
    ///
    /// # Process
    /// 1. Validates string length fits in `u32` including null terminator
    /// 2. Writes all bytes including null terminator using `as_bytes_with_nul()`
    /// 3. Advances stream position by total bytes written
    ///
    /// # Error Handling
    ///
    /// - Returns [`Error::NotValid`] if string length exceeds `u32::MAX - 1`
    /// - C string validity is guaranteed by `CString` type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    /// use std::ffi::CString;
    ///
    /// let mut buffer = vec![0u8; 1024];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// let cstring = CString::new("Test").unwrap();
    /// writer.write(&cstring).unwrap();
    ///
    /// // Contains "Test\0" - 5 bytes total
    /// assert_eq!(&buffer[..5], &[b'T', b'e', b's', b't', b'\0']);
    /// ```
    fn write<T>(mut s: WriteStream<T>, val: &Self) -> Result<()>
    where
        T: AsRef<[u8]> + AsMut<[u8]>,
    {
        if val.count_bytes() > (u32::MAX - 1) as usize {
            return Err(Error::NotValid);
        }

        s.write_exact(val.as_bytes_with_nul())
    }
}
