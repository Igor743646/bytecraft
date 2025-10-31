//! Binary data reader with flexible navigation and type-safe parsing.
//!
//! The `reader` module provides the core functionality for reading binary data
//! in ByteCraft. It offers a comprehensive API for navigating through binary
//! streams, reading primitive types with proper endianness handling, and parsing
//! complex data structures through the `Readable` trait system.
//!
//! # Core Concepts
//!
//! ## ByteReader
//!
//! `ByteReader` is the primary type for reading binary data.
//! It wraps a slice of bytes and allows you to operate on read data
//! after the end of the reader's lifetime, as long as the read data exists.
//!
//! ## Stream Types
//!
//! - [`PeekStream`] - Provides read-only access for previewing data without
//!   changing the reader's position
//! - [`ReadStream`] - Provides read-write access for consuming data and
//!   advancing the reader's position
//!
//! ## Navigation and Positioning
//!
//! ByteCraft offers multiple ways to navigate through data:
//!
//! - **Absolute positioning**: [set_position()](ByteReader::set_position), [seek()](ByteReader::seek), [reset()](ByteReader::reset)
//! - **Relative movement**: [skip()](ByteReader::skip), [rewind()](ByteReader::rewind), [seek()](ByteReader::seek)
//! - **Preview without movement**: [peek()](ByteReader::peek)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use bytecraft::reader::ByteReader;
//!
//! let data = [0x01, 0x02, 0x03, 0x04, 0x05];
//! let mut reader = ByteReader::new(&data[..]);
//!
//! // Read a single byte
//! let first_byte: u8 = reader.read().unwrap();
//! assert_eq!(first_byte, 0x01);
//!
//! // Check current position
//! assert_eq!(reader.position(), 1);
//!
//! // Read multiple bytes
//! let next_bytes = reader.read_bytes(2).unwrap();
//! assert_eq!(next_bytes, &[0x02, 0x03]);
//! ```
//!
//! ## Endianness Handling
//!
//! ```rust
//! use bytecraft::{reader::ByteReader, common::Endian};
//!
//! let data = [0x12, 0x34]; // 0x1234 in big endian
//! let mut reader = ByteReader::with_endian(&data[..], Endian::Big);
//!
//! let value: u16 = reader.read().unwrap();
//! assert_eq!(value, 0x1234);
//!
//! // Switch to little endian
//! reader.set_endian(Endian::Little);
//! reader.reset(); // Go back to start
//!
//! let value: u16 = reader.read().unwrap();
//! assert_eq!(value, 0x3412); // Bytes swapped due to endianness
//! ```
//!
//! ## Complex Data Structures
//!
//! ```rust
//! use bytecraft::{
//!     reader::{ByteReader, ReadStream},
//!     readable::Readable,
//!     error::Result
//! };
//!
//! #[derive(Debug, PartialEq)]
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! impl<'a> Readable<'a> for Point {
//!     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> Result<Self> {
//!         Ok(Point {
//!             x: stream.read()?,
//!             y: stream.read()?,
//!         })
//!     }
//! }
//!
//! let data = [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]; // x=1, y=2
//! let mut reader = ByteReader::new(&data[..]);
//!
//! let point: Point = reader.read().unwrap();
//! assert_eq!(point, Point { x: 1, y: 2 });
//! ```
//!
//! ## Navigation Examples
//!
//! ```rust
//! use bytecraft::{reader::ByteReader, common::{SeekFrom, Endian}};
//!
//! let data = [0u8; 100];
//! let mut reader = ByteReader::new(&data[..]);
//!
//! // Move to specific position
//! reader.set_position(10).unwrap();
//! assert_eq!(reader.position(), 10);
//!
//! // Skip forward
//! reader.skip(5).unwrap();
//! assert_eq!(reader.position(), 15);
//!
//! // Rewind backward
//! reader.rewind(3);
//! assert_eq!(reader.position(), 12);
//!
//! // Seek from different positions
//! reader.seek(SeekFrom::Start(0)).unwrap(); // Beginning
//! reader.seek(SeekFrom::End(-10)).unwrap(); // 10 bytes from end
//! reader.seek(SeekFrom::Current(5)).unwrap(); // 5 bytes forward
//! ```
//!
//! ## String Handling
//!
//! ```rust
//! use bytecraft::reader::ByteReader;
//!
//! // ASCII string reading
//! let ascii_data = b"Hello World";
//! let mut reader = ByteReader::new(&ascii_data[..]);
//! let ascii_string: &str = reader.read_ascii(5).unwrap();
//! assert_eq!(ascii_string, "Hello");
//! assert_eq!(reader.position(), 5);
//!
//! // UTF-8 string reading (using read_vec + manual conversion)
//! let utf8_data = "Հի".as_bytes(); // "Hello" Armenian text
//! let mut reader = ByteReader::new(utf8_data);
//! let bytes = reader.read_vec(4).unwrap(); // "Հի" in UTF-8 is 4 bytes
//! let utf8_string = String::from_utf8(bytes).unwrap();
//! assert_eq!(utf8_string, "Հի");
//! assert_eq!(reader.position(), 4);
//! ```

pub mod peekable;
pub mod readable;

use std::io::Read;

use crate::common::{Endian, SeekFrom};
use crate::error::{Error, Result};
use peekable::Peekable;
use readable::Readable;

/// A versatile binary data reader for parsing structured binary formats.
///
/// `ByteReader` provides a safe and efficient interface for reading binary
/// data from any source that can be represented as a byte slice. It maintains
/// internal state including current position and endianness settings, and
/// provides comprehensive error handling for boundary conditions.
///
/// # Lifetime Parameters
///
/// - `'a`: Lifetime of the underlying data source that represents a slice `&'a [u8]`
///
/// # Examples
///
/// Construct `ByteReader` from commonly used types
///
/// ```rust
/// use std::borrow::Cow;
/// use std::borrow::Borrow;
/// use bytecraft::common::Endian;
/// use bytecraft::reader::ByteReader;
///
/// let slice: [u8; 10] = [0u8; 10];
/// let reader: ByteReader = ByteReader::new(&slice);
///
/// let vec: Vec<u8> = Vec::new();
/// let reader: ByteReader = ByteReader::new(&vec);
///
/// let string: String = String::new();
/// let reader: ByteReader = ByteReader::new(string.as_bytes());
///
/// let vec: Vec<u8> = Vec::new();
/// let cow: Cow<'_, [u8]> = Cow::Borrowed(&vec);
/// let reader: ByteReader = ByteReader::new(cow.borrow());
///
/// let cow: Cow<'_, [u8]> = Cow::Owned(vec);
/// let reader: ByteReader = ByteReader::new(cow.borrow());
/// ```
///
/// See the [module-level documentation](self) for comprehensive examples.
///
#[derive(Hash)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
    endian: Endian,
}

impl<'a> ByteReader<'a> {
    /// Creates a new `ByteReader` with [Endian::Native] parameter.
    ///
    /// Initializes a reader positioned at the beginning of the data with
    /// the system's native byte order setting.
    ///
    /// # Parameters
    ///
    /// - `data`: The source data to read from
    ///
    /// # Returns
    ///
    /// A new `ByteReader` instance positioned at byte 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    /// use bytecraft::common::Endian;
    ///
    /// let data = [1, 2, 3, 4];
    /// let reader = ByteReader::new(&data);
    /// assert_eq!(reader.position(), 0);
    /// assert_eq!(reader.endian(), Endian::Native);
    /// ```
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            endian: Endian::Native,
        }
    }

    /// Creates a new `ByteReader` with specified endianness.
    ///
    /// Similar to [new()](ByteReader::new), but allows explicit control over the byte order
    /// used for multi-byte value parsing.
    ///
    /// # Parameters
    ///
    /// - `data`: The source data to read from
    /// - `endian`: The byte order to use for multi-byte values
    ///
    /// # Returns
    ///
    /// A new `ByteReader` instance with the specified endianness.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{reader::ByteReader, common::Endian};
    ///
    /// let data = [0x12, 0x34];
    /// let reader = ByteReader::with_endian(&data[..], Endian::Big);
    /// ```
    pub fn with_endian(data: &'a [u8], endian: Endian) -> Self {
        Self {
            data,
            pos: 0,
            endian,
        }
    }

    /// Returns the current endianness setting.
    ///
    /// This setting affects how multi-byte values (u16, u32, f32, etc.) are
    /// interpreted when read from the data stream.
    ///
    /// # Returns
    ///
    /// The current [`Endian`] setting of this reader.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{reader::ByteReader, common::Endian};
    ///
    /// let reader = ByteReader::new(&[0u8; 4]);
    /// assert_eq!(reader.endian(), Endian::Native);
    /// ```
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// Sets the endianness for subsequent multi-byte reads.
    ///
    /// Changes how multi-byte values will be interpreted by future read operations.
    /// This setting does not affect the current position or already-read data.
    ///
    /// # Parameters
    ///
    /// - `endian`: The new byte order to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{reader::ByteReader, common::Endian};
    ///
    /// let mut reader = ByteReader::new(&[0x01, 0x00, 0x00, 0x02]);
    /// reader.set_endian(Endian::Little);
    /// let v1: u16 = reader.read().unwrap();
    /// reader.set_endian(Endian::Big);
    /// let v2: u16 = reader.read().unwrap();
    /// assert_eq!((v1, v2), (1, 2));
    /// ```
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    /// Returns the current reading position.
    ///
    /// The position represents the byte offset from the beginning of the data
    /// where the next read operation will occur.
    ///
    /// # Returns
    ///
    /// The current zero-based position in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4]);
    /// assert_eq!(reader.position(), 0);
    ///
    /// reader.read::<u8>().unwrap();
    /// assert_eq!(reader.position(), 1);
    /// ```
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Sets the reading position to an absolute offset.
    ///
    /// Moves the read cursor to the specified byte position. The position must
    /// be within the bounds of the data, or an error will be returned.
    ///
    /// # Parameters
    ///
    /// - `pos`: The zero-based byte position to move to
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the position was successfully set
    /// - [`Error::OutOfBounds`] if the position exceeds data boundaries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4]);
    /// reader.set_position(2).unwrap();
    /// assert_eq!(reader.position(), 2);
    ///
    /// // Attempting to set position beyond data returns error
    /// assert!(reader.set_position(10).is_err());
    /// ```
    pub fn set_position(&mut self, pos: usize) -> Result<()> {
        if pos > self.len() {
            return Err(Error::OutOfBounds {
                pos: self.pos,
                requested: pos,
                len: self.len(),
            });
        }
        self.pos = pos;
        Ok(())
    }

    /// Skips forward by the specified number of bytes with bounds checking.
    ///
    /// Advances the read position by `count` bytes. If the skip would move
    /// beyond the end of the data, an error is returned and the position
    /// remains unchanged.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to skip forward
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the skip was successful
    /// - [`Error::OutOfBounds`] if skipping would exceed data boundaries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// assert_eq!(reader.position(), 0);
    ///
    /// reader.skip(2).unwrap();
    /// assert_eq!(reader.position(), 2);
    ///
    /// // Skipping beyond end returns error
    /// assert!(reader.skip(10).is_err());
    /// assert_eq!(reader.position(), 2);
    /// ```
    pub fn skip(&mut self, count: usize) -> Result<()> {
        match self.pos.checked_add(count) {
            Some(pos) if pos <= self.len() => {
                self.pos = pos;
                Ok(())
            }
            _ => Err(Error::OutOfBounds {
                pos: self.pos,
                requested: count,
                len: self.len(),
            }),
        }
    }

    /// Skips forward by the specified number of bytes, clamping to data end.
    ///
    /// Never fails - if the skip amount exceeds remaining data, position
    /// is set to the end of data.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to skip forward
    ///
    /// # Examples
    /// ```
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// reader.skip_force(10); // More than available
    /// assert_eq!(reader.position(), 5); // Clamped to end
    /// ```
    pub fn skip_force(&mut self, count: usize) {
        self.pos = std::cmp::min(self.pos.saturating_add(count), self.len());
    }

    /// Aligns position upward to the specified power-of-2 boundary with bounds checking.
    ///
    /// # Parameters
    /// - `ALIGNMENT`: The alignment boundary (must be a power of 2)
    ///
    /// # Returns
    /// - `Ok(())` if alignment is successful
    /// - `Error::OutOfBounds` if alignment would exceed data boundaries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut buffer = [0u8; 16];
    /// buffer[5] = 0xFF; // Some data at position 5
    /// let mut reader = ByteReader::new(&mut buffer[..]);
    /// reader.set_position(5);
    ///
    /// // Align to 8-byte boundary
    /// reader.align_up::<8>().unwrap();
    /// assert_eq!(reader.position(), 8);
    ///
    /// reader.set_position(9);
    /// reader.align_up::<8>().unwrap();
    /// assert_eq!(reader.position(), 16);
    /// ```
    pub fn align_up<const ALIGNMENT: usize>(&mut self) -> Result<()> {
        const {
            assert!(ALIGNMENT.is_power_of_two());
        }
        let pos: usize = self.position();
        let pos: usize = (pos + (ALIGNMENT - 1)) & !(ALIGNMENT - 1);
        self.set_position(pos)
    }

    /// Aligns position upward to the specified power-of-2 boundary, clamping to data end.
    ///
    /// Never fails - if alignment would exceed data boundaries, position is clamped.
    ///
    /// # Parameters
    /// - `ALIGNMENT`: The alignment boundary (must be a power of 2)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut reader = ByteReader::new(&mut buffer[..]);
    /// reader.set_position(9);
    ///
    /// // Align to 8-byte boundary (would be 16, but clamped to 10)
    /// reader.align_up_force::<8>();
    /// assert_eq!(reader.position(), 10);
    /// ```
    pub fn align_up_force<const ALIGNMENT: usize>(&mut self) {
        const {
            assert!(ALIGNMENT.is_power_of_two());
        }
        let pos: usize = self.position();
        let pos: usize = (pos + (ALIGNMENT - 1)) & !(ALIGNMENT - 1);
        self.pos = pos.min(self.len());
    }

    /// Aligns position upward to the specified power-of-2 boundary with bounds checking.
    ///
    /// Calculates the next position aligned to the specified boundary and moves the
    /// reader position there. The alignment must be a power of two.
    ///
    /// # Parameters
    ///
    /// - `alignment`: The alignment boundary in bytes (must be a power of 2)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if alignment is successful
    /// - [`Error::NotValid`] if alignment is not a power of 2
    /// - [`Error::OutOfBounds`] if alignment would exceed data boundaries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut buffer = [0u8; 16];
    /// let mut reader = ByteReader::new(&mut buffer[..]);
    /// reader.set_position(5);
    ///
    /// reader.align_up_dynamic(8).unwrap(); // Align to 8-byte boundary
    /// assert_eq!(reader.position(), 8);
    /// ```
    pub fn align_up_dynamic(&mut self, alignment: usize) -> Result<()> {
        if !alignment.is_power_of_two() {
            return Err(Error::NotValid);
        }

        let pos: usize = self.position();
        let pos: usize = (pos + (alignment - 1)) & !(alignment - 1);
        self.set_position(pos)
    }

    /// Rewinds backward by the specified number of bytes with bounds checking.
    ///
    /// Decreases the value of the read position by the number of bytes.
    /// If the rewind goes beyond the beginning of the data,
    /// an error will be returned and the position will remain unchanged.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to move backward
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// reader.set_position(3).unwrap();
    ///
    /// reader.rewind(1);
    /// assert_eq!(reader.position(), 2);
    ///
    /// reader.rewind(5); // More than current position
    /// assert_eq!(reader.position(), 2); // Clamped to beginning
    /// ```
    pub fn rewind(&mut self, count: usize) -> Result<()> {
        match self.pos.checked_sub(count) {
            Some(pos) => {
                self.pos = pos;
                Ok(())
            }
            _ => Err(Error::OutOfBounds {
                pos: self.pos,
                requested: count,
                len: self.len(),
            }),
        }
    }

    /// Rewinds backward by the specified number of bytes, clamping to start.
    ///
    /// Never fails - if the rewind amount exceeds current position, position
    /// is set to the beginning of data.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to move backward
    ///
    /// # Examples
    /// ```
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// reader.set_position(3).unwrap();
    /// reader.rewind_force(10); // More than current position
    /// assert_eq!(reader.position(), 0); // Clamped to start
    /// ```
    pub fn rewind_force(&mut self, count: usize) {
        self.pos = self.pos.saturating_sub(count);
    }

    /// Seeks to a new position based on the specified seek origin.
    ///
    /// Provides flexible positioning similar to file I/O operations. Supports
    /// seeking from the beginning, end, or current position of the data.
    ///
    /// # Parameters
    ///
    /// - `pos`: A [`SeekFrom`] value specifying the seek operation
    ///
    /// # Returns
    ///
    /// - `Ok(new_position)` with the new absolute position
    /// - [`Error::OutOfBounds`] if the seek target is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{reader::ByteReader, common::SeekFrom};
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    ///
    /// // Seek to absolute position
    /// reader.seek(SeekFrom::Start(2)).unwrap();
    /// assert_eq!(reader.position(), 2);
    ///
    /// // Seek relative to current position
    /// reader.seek(SeekFrom::Current(-1)).unwrap();
    /// assert_eq!(reader.position(), 1);
    ///
    /// // Seek from end (negative values move backward from end)
    /// reader.seek(SeekFrom::End(-2)).unwrap();
    /// assert_eq!(reader.position(), 3); // 5-2=3
    /// ```
    pub fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        let new_pos: usize = match pos {
            SeekFrom::Start(n) => n as usize,
            SeekFrom::End(n) if n > 0 => {
                let n: usize = n as usize;
                return Err(Error::OutOfBounds {
                    pos: self.pos,
                    requested: n,
                    len: self.len(),
                });
            }
            SeekFrom::End(n) => {
                let n: usize = (-n) as usize;
                self.len().checked_sub(n).ok_or(Error::OutOfBounds {
                    pos: self.pos,
                    requested: n,
                    len: self.len(),
                })?
            }
            SeekFrom::Current(n) if n > 0 => {
                let n: usize = n as usize;
                self.pos.checked_add(n).ok_or(Error::OutOfBounds {
                    pos: self.pos,
                    requested: n,
                    len: self.len(),
                })?
            }
            SeekFrom::Current(n) => {
                let n: usize = (-n) as usize;
                self.pos.checked_sub(n).ok_or(Error::OutOfBounds {
                    pos: self.pos,
                    requested: n,
                    len: self.len(),
                })?
            }
        };
        self.set_position(new_pos)?;
        Ok(self.pos)
    }

    /// Resets the reader position to the beginning.
    ///
    /// A convenience method that sets the position back to 0, effectively
    /// rewinding to the start of the data stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4]);
    /// reader.read::<u8>().unwrap(); // Move position to 1
    /// assert_eq!(reader.position(), 1);
    ///
    /// reader.reset(); // Back to start
    /// assert_eq!(reader.position(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Returns a reference to the entire underlying data slice.
    ///
    /// Provides direct access to all data, regardless of current position.
    /// Useful for debugging or when you need to access data out of order.
    ///
    /// # Returns
    ///
    /// A slice containing all the underlying data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let data = [1, 2, 3, 4, 5];
    /// let reader = ByteReader::new(&data[..]);
    ///
    /// assert_eq!(reader.as_slice(), &[1, 2, 3, 4, 5]);
    /// ```
    pub fn as_slice(&self) -> &'a [u8] {
        self.data
    }

    /// Consumes the reader and returns the underlying data.
    ///
    /// Takes ownership of the reader and returns the original data container.
    /// This is useful when you're done reading and want to drop reader explicitly.
    ///
    /// # Returns
    ///
    /// The original source data as slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let original_data = vec![1, 2, 3, 4];
    /// let reader = ByteReader::new(&original_data);
    ///
    /// // After processing, get the data back
    /// let data = reader.into_inner();
    /// assert_eq!(data, original_data);
    /// ```
    pub fn into_inner(self) -> &'a [u8] {
        self.data
    }

    /// Returns the total length of the underlying data.
    ///
    /// Provides the size of the entire data stream in bytes, regardless of
    /// current position.
    ///
    /// # Returns
    ///
    /// The total number of bytes in the data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// reader.read::<u8>().unwrap(); // Move position to 1
    /// assert_eq!(reader.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns a slice of the remaining unread data.
    ///
    /// Provides direct access to the portion of data that hasn't been read yet,
    /// starting from the current position to the end.
    ///
    /// # Returns
    ///
    /// A slice containing all remaining unread bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// reader.skip(2).unwrap(); // Skip first 2 bytes
    ///
    /// let remaining = reader.rest_bytes();
    /// assert_eq!(remaining, &[3, 4, 5]);
    /// ```
    pub fn rest_bytes(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }

    /// Returns the number of bytes remaining to be read.
    ///
    /// Calculates how many bytes are left between the current position and
    /// the end of the data.
    ///
    /// # Returns
    ///
    /// The number of unread bytes remaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// assert_eq!(reader.rest_len(), 5);
    ///
    /// reader.skip(2).unwrap();
    /// assert_eq!(reader.rest_len(), 3);
    /// ```
    pub fn rest_len(&self) -> usize {
        self.len() - self.position()
    }

    /// Checks if the underlying data is empty.
    ///
    /// Returns `true` if the data source contains no bytes, `false` otherwise.
    /// This is independent of the current reading position.
    ///
    /// # Returns
    ///
    /// `true` if the data contains zero bytes, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let reader = ByteReader::new(&[][..]);
    /// assert!(reader.is_empty());
    ///
    /// let reader = ByteReader::new(&[1, 2, 3][..]);
    /// assert!(!reader.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    /// Checks if the reader has reached the end of data.
    ///
    /// Returns `true` if the current position is at or beyond the end of the
    /// data, meaning no more bytes can be read.
    ///
    /// # Returns
    ///
    /// `true` if at end of data, `false` if more data is available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2]);
    /// assert!(!reader.is_eof()); // At position 0
    ///
    /// reader.skip(2).unwrap(); // Move to end
    /// assert!(reader.is_eof()); // No more data
    /// ```
    pub fn is_eof(&self) -> bool {
        self.position() >= self.len()
    }

    /// Peeks at exactly `size` bytes without advancing position.
    ///
    /// Returns a reference to `size` bytes starting from the current position
    /// without changing the reader's state. Fails if not enough data is available.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to peek at
    ///
    /// # Returns
    ///
    /// - `Ok(slice)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    /// ```
    fn peek_exact(&self, size: usize) -> Result<&'a [u8]> {
        self.check_bounds(size)?;
        let result: &[u8] = &self.data[self.pos..self.pos + size];
        Ok(result)
    }

    /// Reads exactly `size` bytes and advances position.
    ///
    /// Returns a reference to `size` bytes starting from the current position
    /// and advances the reader's position by `size` bytes.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to read
    ///
    /// # Returns
    ///
    /// - `Ok(slice)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    /// ```
    fn read_exact(&mut self, size: usize) -> Result<&'a [u8]> {
        self.check_bounds(size)?;
        let result: &[u8] = &self.data[self.pos..self.pos + size];
        self.pos += size;
        Ok(result)
    }

    /// Peeks at a value of type `P` without advancing position.
    ///
    /// Uses the [`Peekable`] trait to read a value without changing the reader's
    /// position. This is useful for lookahead operations or validation.
    ///
    /// # Type Parameters
    ///
    /// - `P`: A type that implements [`Peekable`]
    ///
    /// # Returns
    ///
    /// - `Ok(value)` of type `P`
    /// - An error if the value cannot be peeked
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let reader = ByteReader::new(&[0x01, 0x02, 0x03, 0x04]);
    /// let value: u16 = reader.peek().unwrap(); // Peek without consuming
    /// assert_eq!(reader.position(), 0); // Position unchanged
    /// assert_eq!(value, 0x0201);
    /// ```
    pub fn peek<P: Peekable<'a>>(&self) -> Result<P> {
        P::peek(PeekStream { reader: self })
    }

    /// Reads a value of type `R` and advances position.
    ///
    /// Uses the [`Readable`] trait to read and consume a value, advancing the
    /// reader's position accordingly. This is the primary method for reading
    /// structured data.
    ///
    /// # Type Parameters
    ///
    /// - `R`: A type that implements [`Readable`]
    ///
    /// # Returns
    ///
    /// - `Ok(value)` of type `R`
    /// - An error if the value cannot be read
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[0x01, 0x02, 0x03, 0x04]);
    /// assert_eq!(reader.position(), 0);
    ///
    /// let value: u32 = reader.read().unwrap(); // Read and consume
    /// assert_eq!(reader.position(), 4); // Position advanced
    /// assert_eq!(value, 0x04030201);
    /// ```
    pub fn read<R: Readable<'a>>(&mut self) -> Result<R> {
        R::read(ReadStream { reader: self })
    }

    /// Peeks exactly `size` bytes without advancing position.
    ///
    /// Returns a reference to `size` bytes starting from the current position.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to peek
    ///
    /// # Returns
    ///
    /// - `Ok(slice)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// let bytes = reader.peek_bytes(3).unwrap();
    /// assert_eq!(bytes, &[1, 2, 3]);
    /// assert_eq!(reader.position(), 0);
    /// ```
    pub fn peek_bytes(&mut self, size: usize) -> Result<&'a [u8]> {
        self.peek_exact(size)
    }

    /// Reads exactly `size` bytes and advances position.
    ///
    /// Returns a reference to `size` bytes starting from the current position
    /// and advances the reader's position by `size` bytes.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to read
    ///
    /// # Returns
    ///
    /// - `Ok(slice)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// let bytes = reader.read_bytes(3).unwrap();
    /// assert_eq!(bytes, &[1, 2, 3]);
    /// assert_eq!(reader.position(), 3);
    /// ```
    pub fn read_bytes(&mut self, size: usize) -> Result<&'a [u8]> {
        self.read_exact(size)
    }

    /// Reads exactly `size` bytes into a new `Vec<u8>` and advances position.
    ///
    /// Similar to [`read_bytes()`](ByteReader::read_bytes), but returns an owned `Vec<u8>` instead of
    /// a slice reference. Useful when you need owned data or when working with
    /// APIs that require owned collections.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to read
    ///
    /// # Returns
    ///
    /// - `Ok(vec)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(&[1, 2, 3, 4, 5]);
    /// let bytes = reader.read_vec(3).unwrap();
    /// assert_eq!(bytes, vec![1, 2, 3]);
    /// assert_eq!(reader.position(), 3);
    /// ```
    pub fn read_vec(&mut self, size: usize) -> Result<Vec<u8>> {
        self.read_exact(size).map(|bytes| Vec::from(bytes))
    }

    /// Peeks ASCII string as `&str` of specified length and advances position.
    ///
    /// Peeks `size` bytes and attempts to interpret them as an ASCII string slice.
    /// Validates that all bytes are valid ASCII characters (0-127) and UTF-8 sequence.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to read as ASCII
    ///
    /// # Returns
    ///
    /// - `Ok(string)` containing the ASCII text
    /// - [`Error::NotValidAscii`] if non-ASCII bytes are found
    /// - [`Error::NotValidUTF8`] if bytes are not valid UTF-8
    /// - [`Error::InsufficientData`] if not enough bytes are available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(b"Hello World");
    /// let ascii: &str = reader.peek_ascii(5).unwrap();
    /// assert_eq!(ascii, "Hello");
    ///
    /// // Non-ASCII data returns error
    /// let mut reader = ByteReader::new(&[0xC0, 0x80][..]); // Invalid UTF-8
    /// assert!(reader.read_ascii(2).is_err());
    /// ```
    pub fn peek_ascii(&self, size: usize) -> Result<&'a str> {
        match self
            .peek_exact(size)
            .map(|bytes| std::str::from_utf8(bytes))?
        {
            Ok(data) if data.is_ascii() => Ok(data),
            Ok(_) => Err(Error::NotValidAscii),
            Err(err) => Err(Error::NotValidUTF8(err)),
        }
    }

    /// Reads ASCII string as `&str` of specified length and advances position.
    ///
    /// Reads `size` bytes and attempts to interpret them as an ASCII string slice.
    /// Validates that all bytes are valid ASCII characters (0-127) and UTF-8 sequence.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to read as ASCII
    ///
    /// # Returns
    ///
    /// - `Ok(string)` containing the ASCII text
    /// - [`Error::NotValidAscii`] if non-ASCII bytes are found
    /// - [`Error::NotValidUTF8`] if bytes are not valid UTF-8
    /// - [`Error::InsufficientData`] if not enough bytes are available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::reader::ByteReader;
    ///
    /// let mut reader = ByteReader::new(b"Hello World");
    /// let ascii: String = reader.read_ascii(5).unwrap().into();
    /// assert_eq!(ascii, "Hello");
    ///
    /// // Non-ASCII data returns error
    /// let mut reader = ByteReader::new(&[0xC0, 0x80][..]); // Invalid UTF-8
    /// assert!(reader.read_ascii(2).is_err());
    /// ```
    pub fn read_ascii(&mut self, size: usize) -> Result<&'a str> {
        match self
            .read_exact(size)
            .map(|bytes| std::str::from_utf8(bytes))?
        {
            Ok(data) if data.is_ascii() => Ok(data),
            Ok(_) => Err(Error::NotValidAscii),
            Err(err) => Err(Error::NotValidUTF8(err)),
        }
    }

    /// Checks if the requested size is within data bounds.
    ///
    /// Internal helper method that validates whether reading `size` bytes from
    /// the current position would exceed the data boundaries.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to check
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the read would be within bounds
    /// - [`Error::InsufficientData`] if the read would exceed bounds
    #[inline]
    fn check_bounds(&self, size: usize) -> Result<()> {
        match self.pos.checked_add(size) {
            Some(pos) if pos <= self.len() => Ok(()),
            _ => Err(Error::InsufficientData {
                requested: size,
                available: self.len() - self.position(),
            }),
        }
    }
}

/// A read-only view into a `ByteReader` for peeking operations.
///
/// `PeekStream` provides a restricted interface that allows inspecting data
/// without modifying the underlying reader's position. It's primarily used
/// by implementations of the [`Peekable`] trait.
///
/// # Type Parameters
///
/// - `'a`: The lifetime of the borrowed data by `ByteReader`
/// - `'r`: The lifetime of the borrowed `ByteReader` instance
///
/// # Examples
///
/// ```rust
/// use bytecraft::{
///     reader::{ByteReader, PeekStream},
///     peekable::Peekable,
///     error::Result
/// };
///
/// struct MyType(u16);
///
/// impl<'a> Peekable<'a> for MyType {
///     fn peek<'r>(stream: PeekStream<'a, 'r>) -> Result<Self> {
///         let value: u16 = stream.peek()?; // Peek without consuming
///         Ok(MyType(value))
///     }
/// }
/// ```
pub struct PeekStream<'a, 'r>
where
    'a: 'r,
{
    reader: &'r ByteReader<'a>,
}

impl<'a, 'r> PeekStream<'a, 'r> {
    /// Returns the current endianness setting of the underlying reader.
    ///
    /// # Returns
    ///
    /// The [`Endian`] setting that affects multi-byte value interpretation.
    pub fn get_endian(&self) -> Endian {
        self.reader.endian()
    }

    /// Returns a slice containing all remaining unread bytes.
    ///
    /// # Returns
    ///
    /// Slice of bytes &[\[u8]].
    pub fn rest_bytes(&self) -> &[u8] {
        self.reader.rest_bytes()
    }

    /// Returns the number of bytes remaining to be read.
    ///
    /// # Returns
    ///
    /// The length of remaining unread bytes.
    pub fn rest_len(&self) -> usize {
        self.reader.rest_len()
    }

    /// Check if the current position is at or beyond the end of the data
    ///
    /// # Returns
    ///
    /// The `true` if at end of data, `false` if more data is available.
    pub fn is_eof(&self) -> bool {
        self.reader.is_eof()
    }

    /// Peeks at exactly `size` bytes without advancing position.
    ///
    /// Delegates to the underlying reader's `ByteReader::peek_exact` method.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to peek at
    ///
    /// # Returns
    ///
    /// - `Ok(slice)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    pub fn peek_exact(&self, size: usize) -> Result<&[u8]> {
        self.reader.peek_exact(size)
    }

    /// Peeks at a value of type `P` without advancing position.
    ///
    /// Delegates to the underlying reader's [`ByteReader::peek`] method.
    ///
    /// # Type Parameters
    ///
    /// - `P`: A type that implements [`Peekable`]
    ///
    /// # Returns
    ///
    /// - `Ok(value)` of type `P`
    /// - An error if the value cannot be peeked
    pub fn peek<P: Peekable<'a>>(&self) -> Result<P> {
        self.reader.peek::<P>()
    }

    /// Peeks ASCII string as `&str` of specified length and advances position.
    ///
    /// Delegates to the underlying reader's [`ByteReader::peek_ascii`] method.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to peek at
    ///
    /// # Returns
    ///
    /// - `Ok(string)` containing the requested ascii string
    /// - An error if the value cannot be peeked
    pub fn peek_ascii(&self, size: usize) -> Result<&'a str> {
        self.reader.peek_ascii(size)
    }
}

/// A mutable view into a `ByteReader` for reading operations.
///
/// `ReadStream` provides the primary interface for consuming data and advancing
/// the reader's position. It's used by implementations of the [`Readable`] trait.
///
/// # Type Parameters
///
/// - `'a`: The lifetime of the borrowed data by `ByteReader`
/// - `'r`: The lifetime of the borrowed `ByteReader` instance
///
/// # Examples
///
/// ```rust
/// use bytecraft::{
///     reader::{ByteReader, ReadStream},
///     readable::Readable,
///     error::Result
/// };
///
/// struct MyStruct<'a> {
///     id: u32,
///     count: &'a [u8],
/// }
///
/// impl<'a> Readable<'a> for MyStruct<'a> {
///     fn read<'r>(mut stream: ReadStream<'a, 'r>) -> Result<Self> {
///         Ok(MyStruct {
///             id: stream.read()?,      // Read and consume u32
///             count: stream.read_exact(2)?,   // Read and consume 2-byte slice
///         })
///     }
/// }
/// ```
pub struct ReadStream<'a, 'r>
where
    'a: 'r,
{
    reader: &'r mut ByteReader<'a>,
}

impl<'a, 'r> ReadStream<'a, 'r> {
    /// Returns the current endianness setting of the underlying reader.
    ///
    /// # Returns
    ///
    /// The [`Endian`] setting that affects multi-byte value interpretation.
    pub fn get_endian(&self) -> Endian {
        self.reader.endian()
    }

    /// Sets the endianness to the underlying reader.
    pub fn set_endian(&mut self, endian: Endian) {
        self.reader.set_endian(endian);
    }

    /// Returns a slice containing all remaining unread bytes.
    ///
    /// # Returns
    ///
    /// Slice of bytes &[\[u8]].
    pub fn rest_bytes(&self) -> &[u8] {
        self.reader.rest_bytes()
    }

    /// Returns the number of bytes remaining to be read.
    ///
    /// # Returns
    ///
    /// The length of remaining unread bytes.
    pub fn rest_len(&self) -> usize {
        self.reader.rest_len()
    }

    /// Check if the current position is at or beyond the end of the data
    ///
    /// # Returns
    ///
    /// The `true` if at end of data, `false` if more data is available.
    pub fn is_eof(&self) -> bool {
        self.reader.is_eof()
    }

    /// Skips forward by the specified number of bytes with bounds checking.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the skip was successful
    /// - [`Error::OutOfBounds`] if skipping would exceed data boundaries
    pub fn skip(&mut self, count: usize) -> Result<()> {
        self.reader.skip(count)
    }

    /// Skips forward by the specified number of bytes, clamping to data end.
    ///
    /// Never fails - if the skip amount exceeds remaining data, position
    /// is set to the end of data.
    pub fn skip_force(&mut self, count: usize) {
        self.reader.skip_force(count)
    }

    /// Aligns position upward to the specified power-of-2 boundary with bounds checking.
    ///
    /// # Parameters
    /// - `ALIGNMENT`: The alignment boundary (must be a power of 2)
    ///
    /// # Returns
    /// - `Ok(())` if alignment is successful
    /// - `Error::OutOfBounds` if alignment would exceed data boundaries
    pub fn align_up<const ALIGNMENT: usize>(&mut self) -> Result<()> {
        self.reader.align_up::<ALIGNMENT>()
    }

    /// Aligns position upward to the specified power-of-2 boundary, clamping to data end.
    ///
    /// Never fails - if alignment would exceed data boundaries, position is clamped.
    ///
    /// # Parameters
    /// - `ALIGNMENT`: The alignment boundary (must be a power of 2)
    pub fn align_up_force<const ALIGNMENT: usize>(&mut self) {
        self.reader.align_up_force::<ALIGNMENT>()
    }

    /// Aligns position upward to the specified power-of-2 boundary with bounds checking.
    ///
    /// # Parameters
    ///
    /// - `alignment`: The alignment boundary in bytes (must be a power of 2)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if alignment is successful
    /// - [`Error::NotValid`] if alignment is not a power of 2
    /// - [`Error::OutOfBounds`] if alignment would exceed data boundaries
    pub fn align_up_dynamic(&mut self, alignment: usize) -> Result<()> {
        self.reader.align_up_dynamic(alignment)
    }

    /// Peeks at exactly `size` bytes without advancing position.
    ///
    /// Delegates to the underlying reader's `ByteReader::peek_exact` method.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to peek at
    ///
    /// # Returns
    ///
    /// - `Ok(slice)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    pub fn peek_exact(&self, size: usize) -> Result<&[u8]> {
        self.reader.peek_exact(size)
    }

    /// Reads exactly `size` bytes and advances position.
    ///
    /// Delegates to the underlying reader's `ByteReader::read_exact` method.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to read
    ///
    /// # Returns
    ///
    /// - `Ok(slice)` containing the requested bytes
    /// - [`Error::InsufficientData`] if not enough bytes are available
    pub fn read_exact(&mut self, size: usize) -> Result<&'a [u8]>
    where
        'a: 'r,
    {
        self.reader.read_exact(size)
    }

    /// Peeks at a value of type `P` without advancing position.
    ///
    /// Delegates to the underlying reader's [`ByteReader::peek`] method.
    ///
    /// # Type Parameters
    ///
    /// - `P`: A type that implements [`Peekable`]
    ///
    /// # Returns
    ///
    /// - `Ok(value)` of type `P`
    /// - An error if the value cannot be peeked
    pub fn peek<P: Peekable<'a>>(&self) -> Result<P> {
        self.reader.peek::<P>()
    }

    /// Reads a value of type `R` and advances position.
    ///
    /// Delegates to the underlying reader's [`ByteReader::read`] method.
    ///
    /// # Type Parameters
    ///
    /// - `R`: A type that implements [`Readable`]
    ///
    /// # Returns
    ///
    /// - `Ok(value)` of type `R`
    /// - An error if the value cannot be read
    pub fn read<R: Readable<'a>>(&mut self) -> Result<R> {
        self.reader.read::<R>()
    }

    /// Peeks ASCII string as `&str` of specified length and advances position.
    ///
    /// Delegates to the underlying reader's [`ByteReader::peek_ascii`] method.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to peek at
    ///
    /// # Returns
    ///
    /// - `Ok(string)` containing the requested ascii string
    /// - An error if the value cannot be peeked
    pub fn peek_ascii(&self, size: usize) -> Result<&'a str> {
        self.reader.peek_ascii(size)
    }

    /// Reads ASCII string as `&str` of specified length and advances position.
    ///
    /// Delegates to the underlying reader's [`ByteReader::read_ascii`] method.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to read at
    ///
    /// # Returns
    ///
    /// - `Ok(string)` containing the requested ascii string
    /// - An error if the value cannot be read
    pub fn read_ascii(&mut self, size: usize) -> Result<&'a str> {
        self.reader.read_ascii(size)
    }

    /// Reads exactly `size` bytes into a new `Vec<u8>` and advances position.
    ///
    /// Delegates to the underlying reader's [`ByteReader::read_vec`] method.
    ///
    /// # Type Parameters
    ///
    /// - `size`: The number of bytes to read
    ///
    /// # Returns
    ///
    /// - `Ok(vec)` containing the requested bytes
    /// - An error if the value cannot be read
    pub fn read_vec(&mut self, size: usize) -> Result<Vec<u8>> {
        self.reader.read_vec(size)
    }
}

impl Clone for ByteReader<'_> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            pos: self.pos,
            endian: self.endian,
        }
    }
}

impl PartialEq for ByteReader<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.data.as_ref().as_ref().eq(other.data.as_ref().as_ref())
    }
}

impl<'a> From<&'a [u8]> for ByteReader<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a Vec<u8>> for ByteReader<'a> {
    fn from(value: &'a Vec<u8>) -> Self {
        Self::new(value.as_slice())
    }
}

impl<'a> From<&'a String> for ByteReader<'a> {
    fn from(value: &'a String) -> Self {
        Self::new(value.as_bytes())
    }
}

impl Read for ByteReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.rest_len() == 0 {
            return Ok(0);
        }

        let data: &[u8] = self.data.as_ref().as_ref();
        let to_read: usize = std::cmp::min(self.rest_len(), buf.len());
        buf[..to_read].copy_from_slice(&data[self.pos..self.pos + to_read]);
        self.pos += to_read;
        Ok(to_read)
    }
}

impl core::fmt::Debug for ByteReader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ByteReader")
            .field("data", &self.data)
            .field("pos", &self.pos)
            .field("endian", &self.endian)
            .finish()
    }
}

impl core::fmt::Display for ByteReader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n                  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  10 11 12 13 14 15 16 17  18 19 1A 1B 1C 1D 1E 1F")?;

        for (idx, &char) in self.data.iter().enumerate() {
            match (idx % 32, idx % 8) {
                (_, 7) => write!(f, "{:02x}  ", char)?,
                (0, _) => write!(f, "\n{:016x}: {:02x} ", idx / 32, char)?,
                (_, _) => write!(f, "{:02x} ", char)?,
            }
        }

        Ok(())
    }
}
