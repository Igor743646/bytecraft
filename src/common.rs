//! Common types for the ByteCraft library.
//!
//! This module defines the common types required for many operations performed in ByteCraft.
//!
//! # Examples
//!
//! ```
//! use bytecraft::common::Endian;
//! use bytecraft::reader::ByteReader;
//!
//! let data = [0x01, 0x02];
//! let mut reader = ByteReader::new(&data);
//!
//! // set endianness
//! reader.set_endian(Endian::Little);
//! let val: u16 = reader.read().unwrap();
//! assert_eq!(val, 0x0201);
//!
//! ```

use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;

/// Represents the byte order (endianness) used for reading and writing multi-byte values.
///
/// Endianness determines how multi-byte data types (like `u16`, `u32`, `f64`) are
/// stored in memory and interpreted when read from or written to binary data.
///
/// # Examples
///
/// ```
/// use bytecraft::common::Endian;
///
/// // Little endian - least significant byte first
/// let little = Endian::Little;
///
/// // Big endian - most significant byte first  
/// let big = Endian::Big;
///
/// // Native endian - matches the host system's endianness
/// let native = Endian::Native;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Endian {
    /// Little endian byte order.
    ///
    /// In little endian, the least significant byte is stored at the lowest memory address.
    /// For example, the 32-bit value `0x12345678` would be stored as bytes `[0x78, 0x56, 0x34, 0x12]`.
    ///
    /// This is the native byte order on x86, x86-64, and most modern processors.
    ///
    /// # Examples
    ///
    /// ```text
    /// // The value 0x1234 in little endian:
    /// // Memory: [0x34, 0x12]
    /// ```
    Little = 0,

    /// Big endian byte order.
    ///
    /// In big endian, the most significant byte is stored at the lowest memory address.
    /// For example, the 32-bit value `0x12345678` would be stored as bytes `[0x12, 0x34, 0x56, 0x78]`.
    ///
    /// This is the native byte order on some network protocols and older architectures.
    /// Network byte order is always big endian.
    ///
    /// # Examples
    ///
    /// ```text
    /// // The value 0x1234 in big endian:
    /// // Memory: [0x12, 0x34]
    /// ```
    Big = 1,

    /// Native endian byte order.
    ///
    /// Uses the same byte order as the host system. On most modern systems (x86, x86-64, ARM),
    /// this is equivalent to `Little` endian. On some older systems or specific architectures,
    /// it might be equivalent to `Big` endian.
    ///
    /// Use this when you want to work with data in the same format as your system's memory.
    ///
    /// # Platform-specific behavior
    ///
    /// - On x86/x86-64: Equivalent to `Little`
    /// - On most ARM: Equivalent to `Little`  
    /// - On some embedded systems: May be `Big`
    Native = 2,
}

pub const LE: Endian = Endian::Little;
pub const BE: Endian = Endian::Big;
pub const NE: Endian = Endian::Native;

impl Endian {
    pub fn from_utf16_bom(bytes: [u8; 2]) -> Option<Self> {
        match bytes {
            [0xFE, 0xFF] => Some(Endian::Big),
            [0xFF, 0xFE] => Some(Endian::Little),
            _ => None,
        }
    }

    pub fn into_utf16_bom(&self) -> [u8; 2] {
        match self {
            Endian::Big => [0xFE, 0xFF],
            Endian::Little => [0xFF, 0xFE],
            #[cfg(target_endian = "big")]
            Endian::Native => [0xFE, 0xFF],
            #[cfg(target_endian = "little")]
            Endian::Native => [0xFF, 0xFE],
        }
    }

    pub fn from_utf32_bom(bytes: [u8; 4]) -> Option<Self> {
        match bytes {
            [0x00, 0x00, 0xFE, 0xFF] => Some(Endian::Big),
            [0xFF, 0xFE, 0x00, 0x00] => Some(Endian::Little),
            _ => None,
        }
    }

    pub fn into_utf32_bom(&self) -> [u8; 4] {
        match self {
            Endian::Big => [0x00, 0x00, 0xFE, 0xFF],
            Endian::Little => [0xFF, 0xFE, 0x00, 0x00],
            #[cfg(target_endian = "big")]
            Endian::Native => [0x00, 0x00, 0xFE, 0xFF],
            #[cfg(target_endian = "little")]
            Endian::Native => [0xFF, 0xFE, 0x00, 0x00],
        }
    }
}

impl Into<&'static str> for Endian {
    fn into(self) -> &'static str {
        match self {
            Endian::Little => "Endian::Little",
            Endian::Big => "Endian::Big",
            Endian::Native => "Endian::Native",
        }
    }
}

impl Display for Endian {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let val: &'static str = (*self).into();
        core::fmt::Display::fmt(val, f)
    }
}

/// Specifies the position and direction for seeking within a data stream.
///
/// `SeekFrom` defines how to calculate the new position when seeking in a `ByteReader` or `ByteWriter`.
/// It supports three different seeking modes: from the start of the data, from the end,
/// or relative to the current position.
///
/// # Examples
///
/// ```
/// use bytecraft::common::SeekFrom;
/// use bytecraft::reader::ByteReader;
///
/// let data = [0u8; 100];
/// let mut reader = ByteReader::new(&data[..]);
///
/// // Seek to absolute position 10 from start
/// reader.seek(SeekFrom::Start(10)).unwrap();
///
/// // Seek 5 bytes forward from current position
/// reader.seek(SeekFrom::Current(5)).unwrap();
///
/// // The equivalent of previous code
/// reader.skip(5).unwrap();
///
/// // Seek 10 bytes backward from end
/// reader.seek(SeekFrom::End(-10)).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeekFrom {
    /// Seek from the beginning of the data stream.
    ///
    /// The position is calculated as an absolute offset from the start of the data.
    /// The value must be usize and within the bounds of the data.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::common::SeekFrom;
    ///
    /// // Seek to position 42 from the beginning
    /// let seek_pos = SeekFrom::Start(42);
    /// ```
    ///
    Start(usize),

    /// Seek from the end of the data stream.
    ///
    /// The position is calculated relative to the end of the data. Positive values
    /// move beyond the end (which typically results in an error), while negative
    /// values move backward from the end.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::common::SeekFrom;
    ///
    /// // Seek to 10 bytes before the end
    /// let seek_pos = SeekFrom::End(-10);
    ///
    /// // Seek to the very end
    /// let seek_pos = SeekFrom::End(0);
    /// ```
    ///
    /// # Notes
    ///
    /// - `End(0)` positions at the end of data
    /// - `End(-5)` positions 5 bytes before the end
    /// - Positive values typically result in out-of-bounds errors
    End(isize),

    /// Seek relative to the current position.
    ///
    /// The position is calculated by adding the offset to the current position.
    /// Positive values move forward, negative values move backward.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytecraft::common::SeekFrom;
    ///
    /// // Move 10 bytes forward from current position
    /// let seek_pos = SeekFrom::Current(10);
    ///
    /// // Move 5 bytes backward from current position
    /// let seek_pos = SeekFrom::Current(-5);
    ///
    /// // Stay at current position (no movement)
    /// let seek_pos = SeekFrom::Current(0);
    /// ```
    ///
    Current(isize),
}

impl Into<String> for SeekFrom {
    fn into(self) -> String {
        match self {
            SeekFrom::Start(shift) => format!("SeekFrom::Start({})", shift),
            SeekFrom::End(shift) => format!("SeekFrom::End({})", shift),
            SeekFrom::Current(shift) => format!("SeekFrom::Current({})", shift),
        }
    }
}

impl Display for SeekFrom {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let val: String = (*self).into();
        std::fmt::Display::fmt(&val, f)
    }
}
