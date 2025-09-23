# ByteCraft

[![Crates.io](https://img.shields.io/crates/v/bytecraft.svg)](https://crates.io/crates/bytecraft)
[![Documentation](https://docs.rs/bytecraft/badge.svg)](https://docs.rs/bytecraft)
[![License](https://img.shields.io/crates/l/bytecraft.svg)](https://github.com/yourusername/bytecraft/blob/master/LICENSE)
[![Build Status](https://github.com/yourusername/bytecraft/workflows/CI/badge.svg)](https://github.com/yourusername/bytecraft/actions)

A high-performance, type-safe binary data reading and writing library for Rust with compile-time endianness support and zero-cost abstractions.

## Features

### **Powerful Trait System**
- **Readable/Writable traits** - Composable serialization/deserialization
- **Peekable trait** - Non-consuming data inspection
- **Type safety** - Full bounds checking and error handling

### **High Performance**
- **Zero-copy operations** - Direct buffer access
- **Minimal allocations** - Efficient memory usage

### **Safety First**
- **Complete bounds checking** - Prevents buffer overflows
- **Detailed error types** - Informative error messages
- **Memory safety** - Idiomatic Rust safety guarantees
- **Thread safe** - Send + Sync where appropriate

### **Ergonomic API**
- **Intuitive navigation** - Skip, rewind, seek operations
- **Flexible buffer support** - Works with Vec, slices, arrays, custom types
- **Rich ecosystem** - Built-in support for standard types
- **Symmetric design** - Consistent Reader/Writer APIs

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bytecraft = "0.1"
```

## Change Log

See [ChangeLog.md](https://github.com/Igor743646/bytecraft/blob/master/ChangeLog.md).

## Supported Rust Versions

The bytecraft crate now supports 1.89.0 rust version.

## Examples

### Quick start:

```rust
use bytecraft::{ByteReader, ByteWriter, Endian};

// Reading binary data
let data = [0x01, 0x02, 0x03, 0x04];
let mut reader = ByteReader::new(&data[..]);

let first_byte: u8 = reader.read().unwrap();
let word: u16 = reader.read().unwrap();

// Writing binary data
let mut buffer = [0u8; 10];
let mut writer = ByteWriter::new(&mut buffer[..]);

writer.write(&42u32).unwrap();
writer.write_exact("Hello".as_bytes()).unwrap();

// Complex structures
#[derive(Debug, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

impl Readable for Point {
    fn read<T: AsRef<[u8]>>(mut stream: ReadStream<T>) -> Result<Self> {
        Ok(Point {
            x: stream.read()?,
            y: stream.read()?,
        })
    }
}

impl Writable for Point {
    fn write<T>(mut stream: WriteStream<T>, val: &Self) -> Result<()>
    where
        T: AsRef<[u8]> + AsMut<[u8]>,
    {
        stream.write(&val.x)?;
        stream.write(&val.y)?;
        Ok(())
    }
}
```

### Navigation and Positioning:

```rust,no_run
use bytecraft::{ByteReader, SeekFrom};

let mut reader = ByteReader::new(data);
reader.skip(10)?;                    // Skip 10 bytes forward
reader.rewind(5)?;                   // Rewind 5 bytes backward
reader.seek(SeekFrom::Start(0))?;    // Seek to beginning
reader.seek(SeekFrom::End(-4))?;     // Seek 4 bytes from end
```

### Endianness Control:

```rust,no_run
use bytecraft::{ByteReader, Endian};

let mut reader = ByteReader::with_endian(data, Endian::Big);
let value: u32 = reader.read()?; // Big-endian reading

reader.set_endian(Endian::Little);
let value: u32 = reader.read()?; // Little-endian reading
```

### Peeking Operations:

```rust,no_run
// Peek at data without consuming it
let preview: u32 = reader.peek()?;
assert_eq!(reader.position(), 0); // Position unchanged

// Read and consume
let actual: u32 = reader.read()?;
assert_eq!(preview, actual);
```

### Collection Support:

```rust,no_run
// Length-prefixed collections
let vec: Vec<u32> = reader.read()?;     // Reads length + elements
let string: String = reader.read()?;    // Reads length + UTF-8 data
let cstring: CString = reader.read()?;  // Reads null-terminated data
```

## TODO

* `no_std` support.

## Related Projects

Other crates for byte manipulation, reading, writing:
* [byteorder]: A library with read/write operations for primitives in ByteBuffer with support for bit operations
* [bytebuffer]: The crate provides methods for encoding and decoding numbers in either big-endian or little-endian order
* [bytes]: The crate provides a byte buffer structure (Bytes) and functions for working with buffer implementations (Buf, BufMut).
* [binary_util]: Crate provides structures: ByteReader and ByteWriter, which are based on the bytes crate

## License

Bytecraft is under the terms of:

1. [the MIT License](https://opensource.org/license/MIT); and/or
2. [the Apache License v2.0](http://www.apache.org/licenses/LICENSE-2.0).

[Rust]: https://www.rust-lang.org/
[byteorder]: https://docs.rs/byteorder/latest/byteorder/
[bytebuffer]: https://docs.rs/bytebuffer/latest/bytebuffer/struct.ByteBuffer.html
[bytes]: https://docs.rs/bytes/latest/bytes/
[binary_util]: https://docs.rs/binary-util/latest/binary_util/