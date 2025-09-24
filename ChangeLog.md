# Change Log

## 0.2.2 (24-09-2025)

New features:

* Add align functions for ByteReader:
    * align_up::\<ALIGNMENT\>(&mut self) -> Result<()>
    * align_up_force::\<ALIGNMENT\>(&mut self)
    * align_up_dynamic(&mut self, alignment: usize) -> Result<()>

```rust
let data: [u8; 100] = [0u8; 100];
let mut reader: ByteReader = ByteReader::new(&data);

reader.set_position(1)?;
reader.align_up::<4>()?;
assert_eq!(reader.position(), 4);
```

* Add Debug and Display traits implementations for ByteReader:
* Change ByteReader::read_ascii() result value to &str. Add peek_ascii().

## 0.2.1 (24-09-2025)

Bug fix:

* ByteReader::read_ascii() did not advance cursor position. 

## 0.2.0 (24-09-2025)

Breaking changes:

* Changed the ByteReader constructor. Now it only accepts slices. 
This is a necessary solution due to the need to have a lifetime for the data being read.

New features:

* Now you can return slices from the read() function of the Readable trait without having to copy them.

```rust
let data: [u8; 100] = [0u8; 100];

#[derive(Debug, PartialEq)]
struct ParsedData<'a> {
    f1: u32,
    f2: i8,
    hash: &'a [u8],
}

#[derive(Debug, PartialEq)]
struct MyStruct<'a> {
    data: &'a [u8],
    parsed: ParsedData<'a>,
}

impl<'a> Readable<'a> for ParsedData<'a> {
    fn read<'r>(mut s: bytecraft::reader::ReadStream<'a, 'r>) -> Result<Self> {
        let f1: u32 = s.read()?;
        let f2: i8 = s.read()?;
        let hash: &[u8] = s.read_exact(10)?;

        Ok(Self { f1, f2, hash })
    }
}

let v: MyStruct;

{
    let mut reader: ByteReader = ByteReader::new(&data);
    let p: ParsedData = reader.read()?;
    v = MyStruct {
        data: &data,
        parsed: p,
    };
}

assert_eq!(
    v,
    MyStruct {
        data: &data,
        parsed: ParsedData {
            f1: 0,
            f2: 0,
            hash: &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        }
    }
);

Ok(())
```