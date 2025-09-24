use bytecraft::common::{Endian, SeekFrom};
use bytecraft::error::{Error, Result};
use bytecraft::readable::Readable;
use bytecraft::reader::ByteReader;
use bytecraft::writer::writable::Writable;
use bytecraft::writer::ByteWriter;

#[test]
fn constructor() {
    let mut slice: [u8; 10] = [0u8; 10];
    let writer: ByteWriter<_> = ByteWriter::new(&mut slice);
    assert_eq!(writer.endian(), Endian::Native);
    let writer: ByteWriter<_> = ByteWriter::new(slice);
    assert_eq!(writer.endian(), Endian::Native);

    let mut vec: Vec<u8> = Vec::new();
    vec.resize(10, 0);
    let writer: ByteWriter<_> = ByteWriter::new(&mut vec);
    assert_eq!(writer.endian(), Endian::Native);
    let writer: ByteWriter<_> = ByteWriter::new(vec);
    assert_eq!(writer.endian(), Endian::Native);
}

#[test]
fn write_number() -> Result<()> {
    macro_rules! test_number {
        ($Writer:tt, $Type:ty, $Value:expr, $Expected:expr) => {
            match $Writer.write::<$Type>($Value) {
                Ok(()) => {
                    assert_eq!($Writer.as_slice(), &$Expected);
                    $Writer.reset();
                    Ok(())
                }
                Err(err) => Err(err),
            }
        };
    }

    let mut data: [u8; 16] = [0x00u8; 16];
    let mut lwriter: ByteWriter<_> = ByteWriter::with_endian(&mut data, Endian::Little);
    test_number!(
        lwriter,
        u8,
        &0x01,
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        i8,
        &0x01,
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        u16,
        &0x0201,
        [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        i16,
        &0x0201,
        [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        u32,
        &0x04030201,
        [1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        i32,
        &0x04030201,
        [1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        u64,
        &0x0807060504030201,
        [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        i64,
        &0x0807060504030201,
        [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        lwriter,
        u128,
        &0x100F0E0D0C0B0A090807060504030201,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
    )?;
    test_number!(
        lwriter,
        i128,
        &0x100F0E0D0C0B0A090807060504030201,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
    )?;

    let mut data: [u8; 16] = [0x00u8; 16];
    let mut bwriter: ByteWriter<_> = ByteWriter::with_endian(&mut data, Endian::Big);
    test_number!(
        bwriter,
        u8,
        &0x01,
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        i8,
        &0x01,
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        u16,
        &0x0102,
        [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        i16,
        &0x0102,
        [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        u32,
        &0x01020304,
        [1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        i32,
        &0x01020304,
        [1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        u64,
        &0x0102030405060708,
        [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        i64,
        &0x0102030405060708,
        [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0]
    )?;
    test_number!(
        bwriter,
        u128,
        &0x0102030405060708090A0B0C0D0E0F10,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
    )?;
    test_number!(
        bwriter,
        i128,
        &0x0102030405060708090A0B0C0D0E0F10,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
    )?;

    Ok(())
}

#[test]
fn write_chars() -> Result<()> {
    let mut buffer: [u8; 18] = [0u8; 18];
    let mut writer: ByteWriter<_> = ByteWriter::new(&mut buffer);

    for c in "Привет World".chars() {
        writer.write(&c)?;
    }

    assert_eq!("Привет World".as_bytes().len(), 18);
    assert_eq!(buffer, "Привет World".as_bytes());
    Ok(())
}

#[test]
#[should_panic(expected = "The recursion depth limit has been exceeded!")]
fn write_zerosized() {
    struct ZeroSized {}

    static mut REC_DEPTH: u8 = 100;

    impl Writable for ZeroSized {
        fn write<T>(mut s: bytecraft::writer::WriteStream<T>, val: &Self) -> Result<()>
        where
            T: AsRef<[u8]> + AsMut<[u8]>,
        {
            if let Some(new) = unsafe { REC_DEPTH.checked_sub(1) } {
                unsafe { REC_DEPTH = new };
            } else {
                panic!("The recursion depth limit has been exceeded!");
            }

            s.write::<ZeroSized>(val)?;
            Ok(())
        }
    }

    let mut writer: ByteWriter<_> = ByteWriter::new([0x01, 0x02]);
    writer.write(&ZeroSized {}).unwrap();
    assert_eq!(writer.position(), 0);
}

#[test]
fn write_simple() -> Result<()> {
    #[derive(Debug, PartialEq, Eq)]
    pub struct Save(u32, i8);

    impl Writable for Save {
        fn write<T>(mut s: bytecraft::writer::WriteStream<T>, val: &Self) -> Result<()>
        where
            T: AsRef<[u8]> + AsMut<[u8]>,
        {
            s.write(&(val.0, val.1))?;

            Ok(())
        }
    }

    let mut data: [u8; 10] = [0u8; 10];

    let mut writer: ByteWriter<_> = ByteWriter::new(&mut data);

    writer.write(&Save(5, -2))?;
    writer.write(&Save(4, 2))?;

    assert_eq!(
        data,
        [0x05, 0x00, 0x00, 0x00, 0xFE, 0x04, 0x00, 0x00, 0x00, 0x02]
    );

    let mut writer: ByteWriter<_> = ByteWriter::new(&mut data);

    writer.write_bytes(&[0u8; 10])?;
    assert_eq!(writer.as_slice(), [0u8; 10]);

    writer.rewind(2)?;
    assert_eq!(writer.position(), 8);

    assert!(writer.rewind(9).is_err());

    writer.rewind_force(1000);
    assert_eq!(writer.position(), 0);

    writer.seek(SeekFrom::End(-2))?;
    assert_eq!(writer.position(), 8);

    writer.skip(2)?;
    assert!(writer.is_eof());

    Ok(())
}

#[test]
fn write_custom() -> Result<()> {
    #[derive(Debug, PartialEq, Eq)]
    pub struct LableValue(u32, i8, [u16; 12]);

    impl<'a> Readable<'a> for LableValue {
        fn read<'r>(mut s: bytecraft::reader::ReadStream<'a, 'r>) -> Result<Self> {
            let (f1, f2, f3) = s.read::<(u32, i8, [u16; 12])>()?;
            Ok(Self(f1, f2, f3))
        }
    }

    impl Writable for LableValue {
        fn write<T>(mut s: bytecraft::writer::WriteStream<T>, val: &Self) -> Result<()>
        where
            T: AsRef<[u8]> + AsMut<[u8]>,
        {
            s.write(&val.0)?;
            s.write(&val.1)?;

            for v in val.2 {
                s.write(&v)?
            }

            Ok(())
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Label {
        name: String,
        timestamp: u64,
        value: LableValue,
    }

    impl<'a> Readable<'a> for Label {
        fn read<'r>(mut s: bytecraft::reader::ReadStream<'a, 'r>) -> Result<Self> {
            let name_size: u32 = s.read()?;
            let name: &[u8] = s.read_exact(name_size as usize)?;
            let name: String = str::from_utf8(name).map_err(|_| Error::NotValid)?.into();

            let timestamp: u64 = s.read()?;
            let value: LableValue = s.read()?;

            Ok(Self {
                name,
                timestamp,
                value,
            })
        }
    }

    impl Writable for Label {
        fn write<T>(mut s: bytecraft::writer::WriteStream<T>, val: &Self) -> Result<()>
        where
            T: AsRef<[u8]> + AsMut<[u8]>,
        {
            if val.name.len() > u32::MAX as usize {
                return Err(Error::NotValid);
            }

            s.write(&(val.name.len() as u32))?;
            s.write_exact(val.name.as_bytes())?;
            s.write(&val.timestamp)?;
            s.write(&val.value)?;

            Ok(())
        }
    }

    let label: Label = Label {
        name: "Byte".into(),
        timestamp: 17293822569102704640,
        value: LableValue(0x78563412, -5, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
    };

    let mut data: [u8; 45] = [0u8; 45];

    let mut writer: ByteWriter<_> = ByteWriter::with_endian(&mut data, Endian::Big);
    writer.write(&label)?;

    assert_eq!(
        data,
        [
            0x00, 0x00, 0x00, 0x04, b'B', b'y', b't', b'e', 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x78, 0x56, 0x34, 0x12, 0xFB, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00,
            0x04, 0x00, 0x05, 0x00, 0x06, 0x00, 0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x0A, 0x00,
            0x0B, 0x00, 0x0C,
        ]
    );

    let mut lreader: ByteReader = ByteReader::with_endian(&data, Endian::Big);

    let rlabel: Label = lreader.read()?;

    assert_eq!(rlabel, label);

    Ok(())
}
