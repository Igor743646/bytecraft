# Change Log

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