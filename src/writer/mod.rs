//! Binary data writer with flexible navigation and type-safe serialization.
//!
//! The `writer` module provides the core functionality for writing binary data
//! in ByteCraft. It offers a comprehensive API for serializing data to buffers,
//! managing write positions, and handling multi-byte values with proper endianness.
//!
//! # Core Concepts
//!
//! ## ByteWriter
//!
//! `ByteWriter<T>` is the primary type for writing binary data. It wraps any
//! type `T` that implements both `AsRef<[u8]>` and `AsMut<[u8]>`, making it
//! compatible with mutable slices, vectors, and custom buffer types.
//!
//! ## Stream Types
//!
//! - [`WriteStream`] - Provides the interface for `Writable` implementations
//!   to write data and advance the writer's position
//!
//! ## Navigation and Positioning
//!
//! ByteCraft offers multiple ways to navigate through the write buffer:
//!
//! - **Absolute positioning**: [set_position()](ByteWriter::set_position), [seek()](ByteWriter::seek), [reset()](ByteWriter::reset)
//! - **Relative movement**: [skip()](ByteWriter::skip), [rewind()](ByteWriter::rewind), [seek()](ByteWriter::seek)
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use bytecraft::writer::ByteWriter;
//!
//! let mut buffer = [0u8; 10];
//! let mut writer = ByteWriter::new(&mut buffer[..]);
//!
//! // Write a single byte
//! writer.write(&0x42u8).unwrap();
//! assert_eq!(writer.position(), 1);
//!
//! // Write multiple bytes
//! writer.write_bytes(&[0x01, 0x02, 0x03]).unwrap();
//! assert_eq!(writer.position(), 4);
//!
//! // Check the written data
//! assert_eq!(&buffer[..4], &[0x42, 0x01, 0x02, 0x03]);
//! ```
//!
//! ## Endianness Handling
//!
//! ```rust
//! use bytecraft::{writer::ByteWriter, common::Endian};
//!
//! let mut buffer = [0u8; 4];
//! let mut writer = ByteWriter::with_endian(&mut buffer[..], Endian::Big);
//!
//! let value: u16 = 0x1234;
//! writer.write(&value).unwrap();
//!
//! // Big endian: most significant byte first
//! assert_eq!(&writer.as_slice()[..2], &[0x12, 0x34]);
//!
//! // Switch to little endian
//! writer.set_endian(Endian::Little);
//! writer.reset(); // Go back to start
//! writer.write(&value).unwrap();
//!
//! // Little endian: least significant byte first
//! assert_eq!(&writer.as_slice()[..2], &[0x34, 0x12]);
//! ```
//!
//! ## Complex Data Structures
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
//!     x: i32,
//!     y: i32,
//! };
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
//! let point = Point { x: 1, y: 2 };
//! let mut buffer = [0u8; 8];
//! let mut writer = ByteWriter::new(&mut buffer[..]);
//!
//! writer.write(&point).unwrap();
//! // Buffer now contains serialized Point data
//! ```
//!
//! ## Navigation Examples
//!
//! ```rust
//! use bytecraft::{writer::ByteWriter, common::{SeekFrom, Endian}};
//!
//! let mut buffer = [0u8; 100];
//! let mut writer = ByteWriter::new(&mut buffer[..]);
//!
//! // Move to specific position
//! writer.set_position(10).unwrap();
//! assert_eq!(writer.position(), 10);
//!
//! // Skip forward
//! writer.skip(5).unwrap();
//! assert_eq!(writer.position(), 15);
//!
//! // Rewind backward
//! writer.rewind(3).unwrap();
//! assert_eq!(writer.position(), 12);
//!
//! // Safe skip (clamps to buffer end)
//! writer.skip_force(1000); // More than buffer size
//! assert_eq!(writer.position(), 100); // Clamped to end
//!
//! // Seek from different positions
//! writer.seek(SeekFrom::Start(0)).unwrap(); // Beginning
//! writer.seek(SeekFrom::End(-10)).unwrap(); // 10 bytes from end
//! writer.seek(SeekFrom::Current(5)).unwrap(); // 5 bytes forward
//! ```
//!
//! ## String Handling
//!
//! ```rust
//! use bytecraft::writer::ByteWriter;
//! use std::ffi::CString;
//!
//! let mut buffer = vec![0u8; 1024];
//! let mut writer = ByteWriter::new(&mut buffer[..]);
//!
//! // Write Rust string (length-prefixed)
//! let string = "Hello World".to_string();
//! writer.write(&string).unwrap();
//!
//! // Write C string (null-terminated)
//! let cstring = CString::new("C String").unwrap();
//! writer.write(&cstring).unwrap();
//!
//! // Write raw bytes
//! writer.write_bytes(b"Raw data").unwrap();
//! ```
pub mod writable;

use std::io::Write;

use crate::common::{Endian, SeekFrom};
use crate::error::{Error, Result};
use writable::Writable;

/// A versatile binary data writer for serializing structured binary formats.
///
/// `ByteWriter<T>` provides a safe and efficient interface for writing binary
/// data to any buffer that can be represented as a mutable byte slice. It maintains
/// internal state including current position and endianness settings, and
/// provides comprehensive error handling for boundary conditions.
///
/// # Type Parameters
///
/// - `T`: The underlying buffer type that must implement both `AsRef<[u8]>`
///        for reading current data and `AsMut<[u8]>` for writing new data
///
/// # Performance Characteristics
///
/// - **Zero-copy writing**: Direct access to underlying buffer
/// - **Compile-time endianness**: No runtime overhead for endian conversions
/// - **Bounds checking**: All operations are bounds-checked for safety
/// - **Minimal allocations**: Works directly with existing buffers
///
/// # Buffer Requirements
///
/// The buffer type `T` must implement:
/// - `AsRef<[u8]>` - to access existing data for bounds checking
/// - `AsMut<[u8]>` - to write new data to the buffer
///
/// Common compatible types include:
/// - `&mut [u8]` - mutable byte slices
/// - `Vec<u8>` - growable byte vectors
/// - `[u8; N]` - fixed-size arrays
/// - `Box<[u8]>` - heap-allocated slices
///
/// # Examples
///
/// See the [module-level documentation](self) for comprehensive examples.
#[derive(Debug, Eq)]
pub struct ByteWriter<T: AsRef<[u8]> + AsMut<[u8]>> {
    data: T,
    pos: usize,
    endian: Endian,
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ByteWriter<T> {
    /// Creates a new `ByteWriter` with native endianness.
    ///
    /// Initializes a writer positioned at the beginning of the buffer with
    /// the system's native byte order setting.
    ///
    /// # Parameters
    ///
    /// - `data`: The target buffer to write to. Must implement both
    ///           `AsRef<[u8]>` and `AsMut<[u8]>`.
    ///
    /// # Returns
    ///
    /// A new `ByteWriter` instance positioned at byte 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 4];
    /// let writer = ByteWriter::new(&mut buffer[..]);
    /// assert_eq!(writer.position(), 0);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            data,
            pos: 0,
            endian: Endian::Native,
        }
    }

    /// Creates a new `ByteWriter` with specified endianness.
    ///
    /// Similar to [`new()`](ByteWriter::new), but allows explicit control over the byte order
    /// used for multi-byte value serialization.
    ///
    /// # Parameters
    ///
    /// - `data`: The target buffer to write to
    /// - `endian`: The byte order to use for multi-byte values
    ///
    /// # Returns
    ///
    /// A new `ByteWriter` instance with the specified endianness.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{writer::ByteWriter, common::Endian};
    ///
    /// let mut buffer = [0u8; 4];
    /// let writer = ByteWriter::with_endian(&mut buffer[..], Endian::Big);
    /// ```
    pub fn with_endian(data: T, endian: Endian) -> Self {
        Self {
            data,
            pos: 0,
            endian,
        }
    }

    /// Returns the current endianness setting.
    ///
    /// This setting affects how multi-byte values (u16, u32, f32, etc.) are
    /// serialized to the buffer.
    ///
    /// # Returns
    ///
    /// The current [`Endian`] setting of this writer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{writer::ByteWriter, common::Endian};
    ///
    /// let mut buffer = [0u8; 4];
    /// let writer = ByteWriter::new(&mut buffer[..]);
    /// assert_eq!(writer.endian(), Endian::Native);
    /// ```
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// Sets the endianness for subsequent multi-byte writes.
    ///
    /// Changes how multi-byte values will be serialized by future write operations.
    /// This setting does not affect the current position or already-written data.
    ///
    /// # Parameters
    ///
    /// - `endian`: The new byte order to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::{writer::ByteWriter, common::Endian};
    ///
    /// let mut buffer = [0u8; 4];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// writer.set_endian(Endian::Little);
    /// assert_eq!(writer.endian(), Endian::Little);
    /// ```
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    /// Returns the current writing position.
    ///
    /// The position represents the byte offset from the beginning of the buffer
    /// where the next write operation will occur.
    ///
    /// # Returns
    ///
    /// The current zero-based position in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 4];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// assert_eq!(writer.position(), 0);
    ///
    /// writer.write(&0x42u8).unwrap();
    /// assert_eq!(writer.position(), 1);
    /// ```
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Sets the writing position to an absolute offset.
    ///
    /// Moves the write cursor to the specified byte position. The position must
    /// be within the bounds of the buffer, or an error will be returned.
    ///
    /// # Parameters
    ///
    /// - `pos`: The zero-based byte position to move to
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the position was successfully set
    /// - [`Error::OutOfBounds`] if the position exceeds buffer boundaries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// writer.set_position(5).unwrap();
    /// assert_eq!(writer.position(), 5);
    ///
    /// // Attempting to set position beyond buffer returns error
    /// assert!(writer.set_position(15).is_err());
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
    /// Advances the write position by `count` bytes. If the skip would move
    /// beyond the end of the buffer, an error is returned and the position
    /// remains unchanged.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to skip forward
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the skip was successful
    /// - [`Error::OutOfBounds`] if skipping would exceed buffer boundaries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// assert_eq!(writer.position(), 0);
    ///
    /// writer.skip(2).unwrap();
    /// assert_eq!(writer.position(), 2);
    ///
    /// // Skipping beyond end returns error
    /// assert!(writer.skip(15).is_err());
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

    /// Skips forward by the specified number of bytes, clamping to buffer end.
    ///
    /// Advances the write position by `count` bytes, but never beyond the end
    /// of the buffer. If the skip amount exceeds remaining capacity, position
    /// is set to the end of the buffer. This operation never fails.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to skip forward
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// writer.skip_force(15); // More than buffer size
    /// assert_eq!(writer.position(), 10); // Clamped to end
    ///
    /// writer.skip_force(5); // No change - already at end
    /// assert_eq!(writer.position(), 10);
    /// ```
    pub fn skip_force(&mut self, count: usize) {
        self.pos = std::cmp::min(self.pos.saturating_add(count), self.len());
    }

    /// Rewinds backward by the specified number of bytes with bounds checking.
    ///
    /// Moves the write position backward by `count` bytes. If `count` is greater
    /// than the current position, an error is returned and the position
    /// remains unchanged.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to move backward
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the rewind was successful
    /// - [`Error::OutOfBounds`] if rewinding would go before buffer start
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// writer.set_position(5).unwrap();
    ///
    /// writer.rewind(2).unwrap();
    /// assert_eq!(writer.position(), 3);
    ///
    /// // Rewinding beyond start returns error
    /// assert!(writer.rewind(10).is_err());
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
    /// Moves the write position backward by `count` bytes. If `count` is greater
    /// than the current position, the position is set to 0 (beginning of buffer).
    /// This operation never fails.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of bytes to move backward
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// writer.set_position(5).unwrap();
    ///
    /// writer.rewind_force(2);
    /// assert_eq!(writer.position(), 3);
    ///
    /// writer.rewind_force(10); // More than current position
    /// assert_eq!(writer.position(), 0); // Clamped to beginning
    /// ```
    pub fn rewind_force(&mut self, count: usize) {
        self.pos = self.pos.saturating_sub(count);
    }

    /// Seeks to a new position based on the specified seek origin.
    ///
    /// Provides flexible positioning similar to file I/O operations. Supports
    /// seeking from the beginning, end, or current position of the buffer.
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
    /// use bytecraft::{writer::ByteWriter, common::SeekFrom};
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// // Seek to absolute position
    /// writer.seek(SeekFrom::Start(2)).unwrap();
    /// assert_eq!(writer.position(), 2);
    ///
    /// // Seek relative to current position
    /// writer.seek(SeekFrom::Current(3)).unwrap();
    /// assert_eq!(writer.position(), 5);
    ///
    /// // Seek from end (negative values move backward from end)
    /// writer.seek(SeekFrom::End(-2)).unwrap();
    /// assert_eq!(writer.position(), 8); // 10-2=8
    /// ```
    pub fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        let new_pos: usize = match pos {
            SeekFrom::Start(n) => n,
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

    /// Resets the writer position to the beginning.
    ///
    /// A convenience method that sets the position back to 0, effectively
    /// rewinding to the start of the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// writer.write(&0x42u8).unwrap(); // Move position to 1
    /// assert_eq!(writer.position(), 1);
    ///
    /// writer.reset(); // Back to start
    /// assert_eq!(writer.position(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Returns a reference to the entire underlying buffer slice.
    ///
    /// Provides direct access to all data in the buffer, regardless of current position.
    /// Useful for debugging or when you need to access data out of order.
    ///
    /// # Returns
    ///
    /// A slice containing all the underlying buffer data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [1, 2, 3, 4, 5];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// // Modify some data
    /// writer.write(&0xFFu8).unwrap();
    ///
    /// // View entire buffer
    /// assert_eq!(writer.as_slice(), &[0xFF, 2, 3, 4, 5]);
    /// ```
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Returns a mutable reference to the entire underlying buffer slice.
    ///
    /// Provides direct mutable access to all data in the buffer, regardless of
    /// current position. Useful for direct buffer manipulation or initialization.
    ///
    /// # Returns
    ///
    /// A mutable slice containing all the underlying buffer data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 5];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// // Initialize entire buffer
    /// writer.as_mut_slice().fill(0xFF);
    /// assert_eq!(writer.as_slice(), &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }

    /// Consumes the writer and returns the underlying buffer.
    ///
    /// Takes ownership of the writer and returns the original buffer container.
    /// This is useful when you're done writing and want to avoid copying data.
    ///
    /// # Returns
    ///
    /// The original buffer container `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let original_buffer = vec![0u8; 10];
    /// let mut writer = ByteWriter::new(original_buffer.clone());
    ///
    /// // After writing, get the buffer back
    /// writer.write(&0x42u8).unwrap();
    /// let buffer = writer.into_inner();
    /// assert_eq!(buffer[0], 0x42);
    /// ```
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Returns the total length of the underlying buffer.
    ///
    /// Provides the size of the entire buffer in bytes, regardless of
    /// current position.
    ///
    /// # Returns
    ///
    /// The total number of bytes in the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let writer = ByteWriter::new(&mut buffer[..]);
    /// assert_eq!(writer.len(), 10);
    /// ```
    pub fn len(&self) -> usize {
        self.data.as_ref().len()
    }

    /// Returns a slice of the remaining unwritten buffer space.
    ///
    /// Provides direct access to the portion of buffer that hasn't been written to yet,
    /// starting from the current position to the end.
    ///
    /// # Returns
    ///
    /// A slice containing all remaining unwritten bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// writer.skip(3).unwrap(); // Skip first 3 bytes
    ///
    /// let remaining = writer.rest_bytes();
    /// assert_eq!(remaining.len(), 7);
    /// ```
    pub fn rest_bytes(&self) -> &[u8] {
        &self.data.as_ref()[self.pos..]
    }

    /// Returns a mutable slice of the remaining unwritten buffer space.
    ///
    /// Provides direct mutable access to the portion of buffer that hasn't been
    /// written to yet, starting from the current position to the end.
    ///
    /// # Returns
    ///
    /// A mutable slice containing all remaining unwritten bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// writer.set_position(2).unwrap();
    ///
    /// // Fill remaining space with a pattern
    /// writer.rest_mut_bytes().fill(0xFF);
    /// assert_eq!(&buffer[..], &[0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    /// ```
    pub fn rest_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data.as_mut()[self.pos..]
    }

    /// Returns the number of bytes remaining available for writing.
    ///
    /// Calculates how many bytes are left between the current position and
    /// the end of the buffer.
    ///
    /// # Returns
    ///
    /// The number of unwritten bytes remaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// assert_eq!(writer.rest_len(), 10);
    ///
    /// writer.skip(3).unwrap();
    /// assert_eq!(writer.rest_len(), 7);
    /// ```
    pub fn rest_len(&self) -> usize {
        self.len() - self.position()
    }

    /// Checks if there's enough space to write the specified number of bytes.
    ///
    /// A convenience method that checks whether writing `size` bytes at the
    /// current position would exceed the buffer boundaries.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to check for availability
    ///
    /// # Returns
    ///
    /// `true` if there's enough space, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// assert!(writer.can_write(5));  // Can write 5 bytes
    /// assert!(writer.can_write(10)); // Can write 10 bytes (exactly)
    /// assert!(!writer.can_write(15)); // Cannot write 15 bytes
    ///
    /// writer.skip(8).unwrap();
    /// assert!(writer.can_write(2));  // Can write 2 bytes
    /// assert!(!writer.can_write(3)); // Cannot write 3 bytes
    /// ```
    pub fn can_write(&self, size: usize) -> bool {
        self.position() + size <= self.len()
    }

    /// Checks if the underlying buffer is empty.
    ///
    /// Returns `true` if the buffer contains no bytes, `false` otherwise.
    /// This is independent of the current writing position.
    ///
    /// # Returns
    ///
    /// `true` if the buffer contains zero bytes, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut empty_buffer: [u8; 0] = [];
    /// let writer = ByteWriter::new(&mut empty_buffer[..]);
    /// assert!(writer.is_empty());
    ///
    /// let mut buffer = [1, 2, 3];
    /// let writer = ByteWriter::new(&mut buffer[..]);
    /// assert!(!writer.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    /// Checks if the writer has reached the end of the buffer.
    ///
    /// Returns `true` if the current position is at or beyond the end of the
    /// buffer, meaning no more bytes can be written.
    ///
    /// # Returns
    ///
    /// `true` if at end of buffer, `false` if more space is available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 5];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    /// assert!(!writer.is_eof()); // At position 0
    ///
    /// writer.skip(5).unwrap(); // Move to end
    /// assert!(writer.is_eof()); // No more space
    /// ```
    pub fn is_eof(&self) -> bool {
        self.position() >= self.len()
    }

    /// Writes exactly `bytes` to the buffer and advances position.
    ///
    /// Copies the provided byte slice directly to the buffer at the current
    /// position and advances the writer's position by the number of bytes written.
    ///
    /// # Parameters
    ///
    /// - `bytes`: The bytes to write to the buffer
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write was successful
    /// - [`Error::InsufficientData`] if not enough buffer space is available
    fn write_exact(&mut self, bytes: &[u8]) -> Result<()> {
        let size: usize = bytes.len();
        self.check_bounds(size)?;
        let data: &mut [u8] = self.data.as_mut();
        data[self.pos..self.pos + size].copy_from_slice(bytes);
        self.pos += size;
        Ok(())
    }

    /// Writes a value of type `W` and advances position.
    ///
    /// Uses the [`Writable`] trait to serialize and write a value, advancing the
    /// writer's position accordingly. This is the primary method for writing
    /// structured data.
    ///
    /// # Type Parameters
    ///
    /// - `W`: A type that implements [`Writable`]
    ///
    /// # Parameters
    ///
    /// - `val`: A reference to the value to write
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the value was successfully written
    /// - An error if the value cannot be written or buffer space is insufficient
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// writer.write(&42u32).unwrap(); // Write using Writable implementation
    /// assert_eq!(writer.position(), 4);
    /// ```
    pub fn write<W: Writable>(&mut self, val: &W) -> Result<()> {
        W::write(WriteStream { writer: self }, val)
    }

    /// Writes exactly `bytes` to the buffer and advances position.
    ///
    /// A public convenience method equivalent to `write_exact()` for writing
    /// raw byte sequences.
    ///
    /// # Parameters
    ///
    /// - `bytes`: The bytes to write to the buffer
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write was successful
    /// - [`Error::InsufficientData`] if not enough buffer space is available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bytecraft::writer::ByteWriter;
    ///
    /// let mut buffer = [0u8; 5];
    /// let mut writer = ByteWriter::new(&mut buffer[..]);
    ///
    /// writer.write_bytes(&[0x01, 0x02, 0x03]).unwrap();
    /// assert_eq!(writer.position(), 3);
    /// assert_eq!(&buffer[..3], &[0x01, 0x02, 0x03]);
    /// ```
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_exact(bytes)
    }

    /// Checks if the requested write size is within buffer bounds.
    ///
    /// Internal helper method that validates whether writing `size` bytes from
    /// the current position would exceed the buffer boundaries.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes to check
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the write would be within bounds
    /// - [`Error::InsufficientData`] if the write would exceed bounds
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

/// A mutable view into a `ByteWriter` for writing operations.
///
/// `WriteStream` provides the primary interface for serializing data and advancing
/// the writer's position. It's used by implementations of the [`Writable`] trait.
///
/// # Type Parameters
///
/// - `'a`: The lifetime of the borrowed `ByteWriter`
/// - `T`: The underlying buffer type (same as in `ByteWriter<T>`)
///
/// # Examples
///
/// ```rust
/// use bytecraft::{
///     writer::{ByteWriter, WriteStream},
///     writable::Writable,
///     error::Result
/// };
///
/// struct MyStruct {
///     id: u32,
///     count: u16,
/// }
///
/// impl Writable for MyStruct {
///     fn write<T>(mut stream: WriteStream<T>, val: &Self) -> Result<()>
///     where
///         T: AsRef<[u8]> + AsMut<[u8]>,
///     {
///         stream.write(&val.id)?;      // Write and consume u32
///         stream.write(&val.count)?;   // Write and consume u16
///         Ok(())
///     }
/// }
/// ```
pub struct WriteStream<'a, T: AsRef<[u8]> + AsMut<[u8]>> {
    writer: &'a mut ByteWriter<T>,
}

impl<'a, T: AsRef<[u8]> + AsMut<[u8]>> WriteStream<'a, T> {
    /// Returns the current endianness setting of the underlying writer.
    ///
    /// # Returns
    ///
    /// The [`Endian`] setting that affects multi-byte value interpretation.
    pub fn get_endian(&self) -> Endian {
        self.writer.endian()
    }

    /// Check if the current position is at or beyond the end of the buffer
    ///
    /// # Returns
    ///
    /// The `true` if at end of buffer, `false` if more capacity is available.
    pub fn is_eof(&self) -> bool {
        self.writer.is_eof()
    }

    /// Skips forward by the specified number of bytes with bounds checking.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the skip was successful
    /// - [`Error::OutOfBounds`] if skipping would exceed buffer boundaries
    pub fn skip(&mut self, count: usize) -> Result<()> {
        self.writer.skip(count)
    }

    /// Skips forward by the specified number of bytes, clamping to buffer end.
    ///
    /// Never fails - if the skip amount exceeds remaining capacity, position
    /// is set to the end of data.
    pub fn skip_force(&mut self, count: usize) {
        self.writer.skip_force(count)
    }

    /// Writes exactly `size` bytes and advances position.
    ///
    /// Delegates to the underlying writer's `ByteWriter::write_exact` method.
    ///
    /// # Parameters
    ///
    /// - `bytes`: The bytes to write
    ///
    /// # Returns
    ///
    /// - `Ok(())`
    /// - [`Error::InsufficientData`] if not enough capacity are available
    pub fn write_exact(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_exact(bytes)
    }

    /// Writes a value of type `W` and advances position.
    ///
    /// Delegates to the underlying writer's [`ByteWriter::write`] method.
    ///
    /// # Type Parameters
    ///
    /// - `W`: A type that implements [`Writable`]
    ///
    /// # Parameters
    ///
    /// - `val`: A reference to the value to write
    ///
    /// # Returns
    ///
    /// - `Ok(())`
    /// - An error if the value cannot be written
    pub fn write<W: Writable>(&mut self, val: &W) -> Result<()> {
        self.writer.write::<W>(val)
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]> + Clone> Clone for ByteWriter<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            pos: self.pos,
            endian: self.endian,
        }
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> PartialEq for ByteWriter<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data.as_ref().eq(other.data.as_ref())
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> From<T> for ByteWriter<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]> + std::hash::Hash> std::hash::Hash for ByteWriter<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Write for ByteWriter<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.rest_len() == 0 {
            return Ok(0);
        }

        let to_write: usize = std::cmp::min(self.rest_len(), buf.len());
        let data: &mut [u8] = self.data.as_mut();
        data[self.pos..self.pos + to_write].copy_from_slice(&buf[..to_write]);
        self.pos += to_write;
        Ok(to_write)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
