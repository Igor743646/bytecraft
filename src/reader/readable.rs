//! Traits and implementations for reading structured binary data.
//!
//! The `readable` module provides the core trait system that enables ByteCraft
//! to read complex data structures from binary streams. It defines how types
//! should be deserialized from byte sequences and includes implementations
//! for common Rust types.
//!
//! # Core Concepts
//!
//! ## Readable Trait
//!
//! The [`Readable`] trait is the foundation of ByteCraft's deserialization system.
//! Any type that implements `Readable` can be read from a [`ReadStream`] using
//! the [ByteReader::read()](crate::reader::ByteReader::read) method.
//!
//! ## Composition and Nesting
//!
//! `Readable` implementations can compose other `Readable` types, enabling
//! complex nested structures:
//!
//! ```rust
//! use bytecraft::{
//!     reader::{ByteReader, ReadStream},
//!     readable::Readable,
//!     error::Result
//! };
//!
//! #[derive(Debug, PartialEq)]
//! struct Header {
//!     magic: u32,
//!     version: u16,
//! };
//!
//! impl<'a> Readable<'a> for Header {
//!     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> Result<Self> {
//!         Ok(Header {
//!             magic: stream.read()?,    // Reads u32
//!             version: stream.read()?,  // Reads u16
//!         })
//!     }
//! }
//!
//! #[derive(Debug, PartialEq)]
//! struct File {
//!     header: Header,      // Nested Readable type
//!     data: Vec<u8>,       // Another Readable type
//! };
//!
//! impl<'a> Readable<'a> for File {
//!     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> Result<Self> {
//!         let header: Header = stream.read()?;  // Composes Header
//!         let data_len: u32 = stream.read()?;   // Reads length
//!         let data = stream.read_vec(data_len as usize)?; // Reads data
//!         Ok(File { header, data })
//!     }
//! }
//! ```
//!
//! # Built-in Implementations
//!
//! ByteCraft provides `Readable` implementations for many standard types:
//!
//! ## Primitive Types
//!
//! All numeric primitives support endianness-aware reading:
//!
//! ```rust
//! use bytecraft::{reader::ByteReader, common::Endian};
//!
//! let data = [0x12, 0x34, 0x56, 0x78];
//! let mut reader = ByteReader::with_endian(&data[..], Endian::Big);
//!
//! let value: u32 = reader.read().unwrap();
//! assert_eq!(value, 0x12345678);
//! ```
//!
//! ## Arrays
//!
//! Fixed-size arrays where elements implement `Readable`:
//!
//! ```rust
//! use bytecraft::reader::ByteReader;
//!
//! let data = [1u8, 2, 3, 4];
//! let mut reader = ByteReader::new(&data[..]);
//!
//! let array: [u16; 2] = reader.read().unwrap();
//! assert_eq!(array, [0x0201, 0x0403]);
//! ```
//!
//! ## Tuples
//!
//! Tuples up to 13 elements support structured reading:
//!
//! ```rust
//! use bytecraft::reader::ByteReader;
//!
//! let data = [1u8, 2, 3, 4];
//! let mut reader = ByteReader::new(&data[..]);
//!
//! let tuple: (u8, u16) = reader.read().unwrap();
//! assert_eq!(tuple, (1, 0x0302u16)); // Assuming little endian
//! ```
//!
//! # Safety Considerations
//!
//! - Memory safety is maintained through proper bounds checking
//!
//! # Custom Implementation Guide
//!
//! To make your type `Readable`, implement the trait:
//!
//! ```rust
//! use bytecraft::{
//!     reader::{ByteReader, ReadStream},
//!     readable::Readable
//! };
//!
//! #[derive(Debug, PartialEq)]
//! struct Point {
//!     x: f32,
//!     y: f32,
//! }
//!
//! impl<'a> Readable<'a> for Point {
//!     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> bytecraft::error::Result<Self> {
//!         Ok(Point {
//!             x: stream.read()?,
//!             y: stream.read()?,
//!         })
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let data = [0u8, 0, 128, 63, 0, 0, 0, 64]; // Represents (1.0, 2.0)
//!     let mut reader = ByteReader::new(&data[..]);
//!     let point: Point = reader.read()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling in Custom Types
//!
//! Custom implementations should propagate errors appropriately:
//!
//! ```rust
//! use bytecraft::{
//!     reader::ReadStream,
//!     readable::Readable,
//!     error::{Result, Error}
//! };
//!
//! #[derive(Debug)]
//! struct ValidatedData {
//!     value: u32,
//! }
//!
//! impl<'a> Readable<'a> for ValidatedData {
//!     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> Result<Self> {
//!         let value: u32 = stream.read()?;
//!         
//!         // Custom validation
//!         if value > 1000 {
//!             return Err(Error::NotValid);
//!         }
//!         
//!         Ok(ValidatedData { value })
//!     }
//! }
//! ```

use core::mem::MaybeUninit;
use std::ffi::CString;

use crate::common::Endian;
use crate::error::{Error, Result};
use crate::reader::ReadStream;

/// A trait for types that can be read from a binary stream.
///
/// The `Readable` trait defines how a type should be deserialized from a
/// sequence of bytes. Implementations receive a [`ReadStream`] that provides
/// access to the underlying data and handles position management.
///
/// # Safety
///
/// Implementations must ensure that:
/// - All reads are properly bounds-checked
/// - Memory safety is maintained
/// - Errors are propagated appropriately
/// - Calling read does not cause recursion
///
/// # Composition
///
/// `Readable` implementations can compose other `Readable` types, enabling
/// complex nested structures to be read seamlessly.
///
/// # Examples
///
/// ## Simple Implementation
///
/// ```rust
/// use bytecraft::{
///     reader::ReadStream,
///     readable::Readable,
///     error::Result
/// };
///
/// struct SimpleStruct {
///     id: u32,
///     flag: bool,
/// }
///
/// impl<'a> Readable<'a> for SimpleStruct {
///     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> Result<Self> {
///         Ok(SimpleStruct {
///             id: stream.read()?,
///             flag: stream.read()?,
///         })
///     }
/// }
/// ```
///
/// ## Complex Implementation with Custom Logic
///
/// ```rust
/// use bytecraft::{
///     reader::ReadStream,
///     readable::Readable,
///     error::{Result, Error}
/// };
///
/// struct LengthPrefixedString(String);
///
/// impl<'a> Readable<'a> for LengthPrefixedString {
///     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> Result<Self> {
///         let len: u32 = stream.read()?;
///         let bytes: &[u8] = stream.read_exact(len as usize)?;
///         let data = std::str::from_utf8(bytes)
///             .map_err(|e| Error::NotValidUTF8(e))?;
///         Ok(LengthPrefixedString(data.to_string()))
///     }
/// }
/// ```
///
/// # See Also
///
/// - [`crate::reader::ByteReader::read()`] - The primary method for reading `Readable` types
/// - [`crate::reader::ReadStream`] - The stream type provided to implementations
pub trait Readable<'a>: Sized {
    /// Reads a value of this type from the provided stream.
    ///
    /// This method is called by [ByteReader::read()](super::ByteReader::read) to deserialize values.
    /// The implementation should consume exactly the number of bytes required
    /// for this type and advance the stream position accordingly.
    ///
    /// # Parameters
    ///
    /// - `stream`: A [`ReadStream`] providing access to the binary data
    ///
    /// # Returns
    ///
    /// - `Ok(value)` containing the deserialized value
    /// - An error if deserialization fails or data is invalid
    ///
    /// # Implementation Notes
    ///
    /// - Use `stream.read::<T>()?` to read other `Readable` types
    /// - Use `stream.read_exact(size)?` for raw byte access
    /// - Consider endianness when reading multi-byte values
    /// - Handle errors appropriately with meaningful error types
    fn read<'r>(s: ReadStream<'a, 'r>) -> Result<Self>;
}

macro_rules! impl_number {
    ($Type:ty) => {
        impl<'a> Readable<'a> for $Type {
            /// Reads a numeric value with proper endianness handling.
            ///
            /// # Process
            /// 1. Reads exactly `size_of::<$Type>()` bytes from the stream
            /// 2. Converts the value from bytes using appropriate endian conversion
            /// 3. Advances the stream position by the number of bytes written
            ///
            /// # Endianness Handling
            ///
            /// The conversion method depends on the stream's endianness setting:
            /// - [`Endian::Little`]: Uses `from_le_bytes()`
            /// - [`Endian::Big`]: Uses `from_be_bytes()`
            /// - [`Endian::Native`]: Uses `from_ne_bytes()`
            fn read(mut s: ReadStream) -> Result<Self> {
                let data: &[u8] = s.read_exact(size_of::<$Type>())?;
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

impl<'a> Readable<'a> for bool {
    /// Reads a byte ([u8]) and converts it to bool.
    /// - 0 - false
    /// - 1 - true
    ///
    /// Returns error [Error::NotValid] if value is not 0 or 1.
    fn read(mut s: ReadStream) -> Result<Self> {
        match s.read::<u8>() {
            Ok(0) => Ok(false),
            Ok(1) => Ok(true),
            Ok(_) => Err(Error::NotValid),
            Err(err) => Err(err),
        }
    }
}

/// Implementation of `Readable` for fixed-size arrays.
///
/// This implementation allows reading arrays where:
/// - Elements implement `Readable`
///
/// # Safety
///
/// This implementation uses `unsafe` code because:
/// - `MaybeUninit` is used for uninitialized array storage
/// - Elements are written directly to uninitialized memory
/// - The array is marked as initialized after all elements are written
///
/// This implementation uses `unsafe` code, but maintains memory safety through:
/// - `MaybeUninit<[T; N]>` for uninitialized storage
/// - Each element is written exactly once before array is marked initialized
/// - If any element fails to read, we properly drop already initialized elements
/// - `assume_init()` is only called when all elements are guaranteed initialized
///
/// # Examples
///
/// ```rust
/// use bytecraft::reader::ByteReader;
///
/// // Reading array of primitives
/// let data = [1u8, 2, 3, 4, 5];
/// let mut reader = ByteReader::new(&data[..]);
/// let array: [u8; 3] = reader.read().unwrap();
/// assert_eq!(array, [1, 2, 3]);
///
/// // Reading array of complex types (if they implement Readable)
/// // let points: [Point; 2] = reader.read().unwrap();
/// ```
impl<'a, T: Readable<'a>, const N: usize> Readable<'a> for [T; N] {
    /// Reads a fixed-size array by reading each element sequentially.
    ///
    /// # Error Handling
    ///
    /// If any element fails to read, the error is propagated immediately.
    /// Elements read before the error drops from uninitialized memory.
    ///
    /// # Performance
    ///
    /// - Time complexity: O(N) where N is array length
    /// - Space complexity: O(N) for the result array
    /// - No additional allocations beyond what elements require
    fn read<'r>(mut s: ReadStream<'a, 'r>) -> Result<Self> {
        let mut data: MaybeUninit<[T; N]> = MaybeUninit::uninit();

        let mut initialized_count: usize = 0;
        let data_ptr: *mut [T; N] = data.as_mut_ptr();

        // Safety: We handle cleanup in case of errors
        struct CleanupGuard<T, const N: usize> {
            data_ptr: *mut [T; N],
            initialized_count: usize,
        }

        impl<T, const N: usize> Drop for CleanupGuard<T, N> {
            fn drop(&mut self) {
                // Drop already initialized elements in reverse order
                for i in (0..self.initialized_count).rev() {
                    unsafe {
                        self.data_ptr.cast::<T>().add(i).drop_in_place();
                    }
                }
            }
        }

        let mut guard: CleanupGuard<T, N> = CleanupGuard {
            data_ptr,
            initialized_count: 0,
        };

        for i in 0..N {
            match s.read() {
                Ok(value) => {
                    unsafe {
                        // Write the value to uninitialized memory
                        data_ptr.cast::<T>().add(i).write(value);
                    }
                    initialized_count += 1;
                    guard.initialized_count = initialized_count;
                }
                Err(e) => {
                    // Guard will clean up already initialized elements
                    return Err(e);
                }
            }
        }

        core::mem::forget(guard);

        Ok(unsafe { data.assume_init() })
    }
}

macro_rules! impl_tupple {
    ($($Types:tt)+) => {
        /// Reads a tuple by reading each field sequentially.
        ///
        /// # Process
        /// 1. Reads each type parameter in order using `stream.read()`
        /// 2. Advances the stream position by each element read
        ///
        /// # Element Order
        ///
        /// Elements are read in the same order as they appear in the tuple
        /// type definition, from left to right.
        ///
        /// # Error Propagation
        ///
        /// If any element fails to read, the error is immediately
        /// propagated and subsequent elements are not read.
        impl<'a, $($Types : Readable<'a>),+> Readable<'a> for ($($Types ,)+) {
            fn read<'r>(mut s: ReadStream<'a, 'r>) -> Result<Self> {
                Ok(($(s.read::<$Types>()?,)+))
            }
        }
    };
}

// Tuple implementations from 1 to 13 elements
impl_tupple!(T0);
impl_tupple!(T0 T1);
impl_tupple!(T0 T1 T2);
impl_tupple!(T0 T1 T2 T3);
impl_tupple!(T0 T1 T2 T3 T4);
impl_tupple!(T0 T1 T2 T3 T4 T5);
impl_tupple!(T0 T1 T2 T3 T4 T5 T6);
impl_tupple!(T0 T1 T2 T3 T4 T5 T6 T7);
impl_tupple!(T0 T1 T2 T3 T4 T5 T6 T7 T8);
impl_tupple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9);
impl_tupple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10);
impl_tupple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11);
impl_tupple!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12);

impl<'a, T: Readable<'a>> Readable<'a> for Vec<T> {
    /// Reads a vector from a length-prefixed binary format.
    ///
    /// # Process
    /// 1. Reads a `u32` length prefix from the stream
    /// 2. Pre-allocates a vector with the specified capacity
    /// 3. Reads each element sequentially using `T::read()`
    /// 4. Returns the populated vector
    ///
    /// # Stream Position
    ///
    /// The stream position is advanced by:
    /// - 4 bytes for the length prefix
    /// - N × sizeof(element) bytes for the elements
    ///
    /// # Element Order
    ///
    /// Elements are read in the same order they were written, maintaining
    /// consistency with the serialization process.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{reader::ByteReader, readable::Readable};
    ///
    /// // Reading a vector of tuples
    /// let data = [
    ///     2u8, 0, 0, 0,  // Length: 2 elements
    ///     1, 2,          // First tuple: (1, 2)
    ///     3, 4           // Second tuple: (3, 4)
    /// ];
    /// let mut reader = ByteReader::new(&data[..]);
    /// let vec: Vec<(u8, u8)> = reader.read().unwrap();
    /// assert_eq!(vec, vec![(1, 2), (3, 4)]);
    /// ```
    ///
    /// # Error Propagation
    ///
    /// If any element fails to read, the error is immediately propagated
    /// and the partially constructed vector is dropped.
    fn read<'r>(mut s: ReadStream<'a, 'r>) -> Result<Self> {
        let size: u32 = s.read()?;

        let mut result: Vec<T> = Vec::with_capacity(size as usize);

        for _ in 0..size {
            let val: T = s.read()?;
            result.push(val);
        }

        Ok(result)
    }
}

impl<'a> Readable<'a> for String {
    /// Reads a string from a length-prefixed UTF-8 binary format.
    ///
    /// # Process
    /// 1. Reads a `u32` length prefix from the stream
    /// 2. Reads the specified number of bytes as a `Vec<u8>`
    /// 3. Converts the byte vector to a `String` with UTF-8 validation
    /// 4. Returns the validated string
    ///
    /// # Stream Position
    ///
    /// The stream position is advanced by:
    /// - 4 bytes for the length prefix
    /// - N bytes for the UTF-8 data
    ///
    /// # UTF-8 Validation
    ///
    /// The implementation uses `String::from_utf8()` which validates that
    /// the input bytes form a valid UTF-8 sequence. Invalid sequences result
    /// in [`Error::NotValidUTF8`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{reader::ByteReader, readable::Readable};
    ///
    /// // Reading Unicode strings
    /// let data = [
    ///     7u8, 0, 0, 0,           // Length: 7 bytes
    ///     226, 156, 133,          // UTF-8 for '✅'
    ///     240, 159, 166, 128      // UTF-8 for '🦀'
    /// ];
    /// let mut reader = ByteReader::new(&data[..]);
    /// let string: String = reader.read().unwrap();
    /// assert_eq!(string, "✅🦀");
    /// ```
    ///
    /// # Error Propagation
    ///
    /// UTF-8 validation errors are wrapped in [`Error::NotValidUTF8`] and
    /// propagated to the caller.
    fn read(mut s: ReadStream) -> Result<Self> {
        let vec: Vec<u8> = s.read()?;
        String::from_utf8(vec).map_err(|err| Error::NotValidUTF8(err.utf8_error()))
    }
}

impl<'a> Readable<'a> for CString {
    /// Reads a C string from a null-terminated binary format.
    ///
    /// # Process
    /// 1. Reads bytes one by one from the stream
    /// 2. Continues until a null terminator (`\0`) is encountered
    /// 3. Validates the byte sequence using `CString::from_vec_with_nul()`
    /// 4. Returns the validated C string
    ///
    /// # Stream Position
    ///
    /// The stream position is advanced by N + 1 bytes, where N is the number
    /// of non-null bytes in the string.
    ///
    /// # Null Terminator Validation
    ///
    /// The implementation uses `CString::from_vec_with_nul()` which validates
    /// that the byte sequence contains exactly one null terminator at the end.
    /// Invalid sequences result in [`Error::Custom`] wrapping the underlying
    /// `FromVecWithNulError`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{reader::ByteReader, readable::Readable};
    /// use std::ffi::CString;
    ///
    /// // Reading C strings with various content
    /// let data = [b't', b'e', b's', b't', b'\0'];
    /// let mut reader = ByteReader::new(&data[..]);
    /// let cstring: CString = reader.read().unwrap();
    /// assert_eq!(cstring.to_str().unwrap(), "test");
    ///
    /// // Reading empty C string
    /// let data = [b'\0'];
    /// let mut reader = ByteReader::new(&data[..]);
    /// let cstring: CString = reader.read().unwrap();
    /// assert_eq!(cstring.to_str().unwrap(), "");
    /// ```
    ///
    /// # Error Propagation
    ///
    /// C string validation errors are wrapped in [`Error::Custom`] and
    /// propagated to the caller. This includes cases like:
    /// - Missing null terminator (stream ends prematurely)
    fn read(mut s: ReadStream) -> Result<Self> {
        let mut result: Vec<u8> = Vec::new();

        loop {
            let b: u8 = s.read()?;

            result.push(b);

            if b == b'\0' {
                break;
            }
        }

        CString::from_vec_with_nul(result).map_err(|err| Error::Custom(Box::new(err)))
    }
}
