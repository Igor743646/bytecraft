//! # ByteCraft
//!
//! **ByteCraft** is a flexible and efficient library for reading and writing binary data in Rust.
//! The library provides a flexible and secure API for parsing complex binary formats, serializing data structures, and working with byte streams.
//!
//! The goal is to create a lightweight, ergonomic and productive library for working with binary data
//!, which combines the best practices from existing solutions and adds missing functionality to solve various problems.
//!
//! ## Goals
//! The library strives to have the following advantages
//!
//! - **Flexible architecture**
//!     - Type composition: Complex structures are built from simple components
//!     - Recursive reading and writing: Types can use other Readable, Writable types inside themselves
//!     - Abstraction of data sources from logic: Data access via interfaces
//!
//! - **Performance**
//!     - Zero overhead: Minimal abstractions without runtime overhead
//!     - Efficient memory management: Minimal allocation and copying
//!
//! - **Security**
//!     - Full border check: Automatic validation of data access
//!     - Idiomatic Rust: Following best security practices
//!     - Informative errors: Detailed error messages
//!
//! - **Ease of use**
//!     - Intuitive API: A readable and intuitive interface
//!     - A rich set of methods: Navigation, search, positioning
//!
//! ## Usefulness
//! This library will be useful for working in many areas
//!
//! - **For developers of binary format parsers**
//!     - Working with network protocols
//!     - Parsing of file formats (PNG, MP3, ELF, etc.)
//!     - Implementation of serializers/deserializers
//!
//! - **For system programmers**
//!     - Working with raw data in embedded systems
//!     - Parsing data from drivers and devices
//!     - no_std environment (with support)
//!
//! - **In GameDev development**
//!     - Work with game formats (assets, save files)
//!     - Real-time network protocols
//!     - Binary serialization of game states
//!
//! - **In scientific and engineering applications**
//!     - Working with scientific data (binary data files)
//!     - Parsing data from instruments and sensors
//!     - Data exchange formats (FITS, HDF5, etc.)
//!
//! ## Main features
//!
//! #### Reading data
//! ```rust
//! use bytecraft::reader::ByteReader;
//! use bytecraft::reader::ReadStream;
//! use bytecraft::readable::Readable;
//! use bytecraft::error::Result;
//!
//! struct MyStruct {
//!     f1: u8,
//!     f2: i32,
//! }
//!
//! impl Readable for MyStruct {
//!     fn read<S: AsRef<[u8]>>(mut s: ReadStream<S>) -> Result<Self> {
//!         let (f1, f2) = s.read::<(u8, i32)>()?;
//!
//!         Ok(MyStruct {f1, f2})
//!     }
//! }
//!
//! let mut reader = ByteReader::new([0x01, 0xFF, 0x00, 0x00, 0x00]);
//! let value: u32 = reader.read().unwrap(); // Direct reading
//! reader.reset();
//! let complex: MyStruct = reader.read().unwrap(); // Composite types
//! ```
//!
//! #### Navigation
//! - `skip()`, `rewind()` - relative navigation
//! - `seek()` - absolute and relative navigation
//! - `peek()` - view without changing position
//!
//! #### Working with dimensions
//! - Automatic border check
//! - Information about the remaining data
//!
//! #### Endianness
//! - Selection of Compile-time and Runtime handlers
//! - Support for Little/Big/Native endian
//!
//! ## Getting Started
//!
//! ```toml
//! [dependencies]
//! bytecraft = "1.0.0"
//! ```
//!
//! ```rust
//! use bytecraft::error::Result;
//! use bytecraft::readable::Readable;
//! use bytecraft::reader::{ByteReader, ReadStream};
//! use bytecraft::writer::writable::Writable;
//! use bytecraft::writer::{ByteWriter, WriteStream};
//!
//! #[derive(Debug, PartialEq)]
//! struct Point {
//!     x: f32,
//!     y: f32,
//! }
//!
//! impl Readable for Point {
//!     fn read<T: AsRef<[u8]>>(mut s: ReadStream<T>) -> Result<Self> {
//!         Ok(Point {
//!             x: s.read()?,
//!             y: s.read()?,
//!         })
//!     }
//! }
//!
//! impl Writable for Point {
//!     fn write<T>(mut s: WriteStream<T>, val: &Self) -> Result<()>
//!     where
//!         T: AsRef<[u8]> + AsMut<[u8]>,
//!     {
//!         s.write(&val.x)?;
//!         s.write(&val.y)?;
//!         Ok(())
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     let original_vec: Vec<Point> = vec![
//!         Point { x: 1.0, y: 2.0 },
//!         Point { x: 3.0, y: 4.0 },
//!         Point { x: 5.0, y: 6.0 },
//!     ];
//!    
//!     let mut buffer: Vec<u8> = vec![0u8; 1024];
//!     let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer[..]);
//!     writer.write(&original_vec)?;
//!    
//!     let mut reader: ByteReader<_> = ByteReader::new(&buffer[..]);
//!     let read_vec: Vec<Point> = reader.read()?;
//!    
//!     assert_eq!(original_vec, read_vec);
//!     Ok(())
//! }
//! ```
//!
//! ## Future development
//!
//! - **Full no_std support** for embedded systems
//!
//! ---

////////////////////////////////////////////////////////////////////////////////

pub mod common;
pub mod error;
pub mod reader;
pub mod writer;

pub use reader::peekable;
pub use reader::readable;
pub use writer::writable;
