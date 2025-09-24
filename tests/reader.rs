use std::borrow::Borrow;
use std::borrow::Cow;

use bytecraft::common::Endian;
use bytecraft::error::Error;
use bytecraft::error::Result;
use bytecraft::peekable::Peekable;
use bytecraft::readable::Readable;
use bytecraft::reader::ByteReader;

#[test]
fn constructor() {
    let slice: [u8; 10] = [0u8; 10];
    let reader: ByteReader = ByteReader::new(&slice);
    assert_eq!(reader.endian(), Endian::Native);

    let vec: Vec<u8> = Vec::new();
    let reader: ByteReader = ByteReader::new(&vec);
    assert_eq!(reader.endian(), Endian::Native);

    let string: String = String::new();
    let reader: ByteReader = ByteReader::new(string.as_bytes());
    assert_eq!(reader.endian(), Endian::Native);

    let vec: Vec<u8> = Vec::new();
    let cow: Cow<'_, [u8]> = Cow::Borrowed(&vec);
    let reader: ByteReader = ByteReader::new(cow.borrow());
    assert_eq!(reader.endian(), Endian::Native);

    let cow: Cow<'_, [u8]> = Cow::Owned(vec);
    let reader: ByteReader = ByteReader::new(cow.borrow());
    assert_eq!(reader.endian(), Endian::Native);
}

#[test]
fn read_number() -> Result<()> {
    let data: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F,
    ];

    macro_rules! test_number {
        ($Reader:tt, $Type:ty, $Result:tt) => {
            match $Reader.read::<$Type>() {
                Ok(val) if val == $Result => {
                    $Reader.reset();
                    Ok(())
                }
                Ok(_) => {
                    $Reader.reset();
                    Err(Error::NotValid)
                }
                Err(err) => Err(err),
            }
        };
    }

    let mut lreader: ByteReader = ByteReader::with_endian(&data, Endian::Little);
    let mut breader: ByteReader = ByteReader::with_endian(&data, Endian::Big);

    test_number!(lreader, u8, 0x00)?;
    test_number!(breader, u8, 0x00)?;
    test_number!(lreader, i8, 0x00)?;
    test_number!(breader, i8, 0x00)?;
    test_number!(lreader, u16, 0x0100)?;
    test_number!(breader, u16, 0x0001)?;
    test_number!(lreader, i16, 0x0100)?;
    test_number!(breader, i16, 0x0001)?;
    test_number!(lreader, u32, 0x03020100)?;
    test_number!(breader, u32, 0x00010203)?;
    test_number!(lreader, i32, 0x03020100)?;
    test_number!(breader, i32, 0x00010203)?;
    test_number!(lreader, u64, 0x0706050403020100)?;
    test_number!(breader, u64, 0x0001020304050607)?;
    test_number!(lreader, i64, 0x0706050403020100)?;
    test_number!(breader, i64, 0x0001020304050607)?;
    test_number!(lreader, u128, 0x0F0E0D0C0B0A09080706050403020100)?;
    test_number!(breader, u128, 0x000102030405060708090A0B0C0D0E0F)?;
    test_number!(lreader, i128, 0x0F0E0D0C0B0A09080706050403020100)?;
    test_number!(breader, i128, 0x000102030405060708090A0B0C0D0E0F)?;

    Ok(())
}

#[test]
#[should_panic(expected = "The recursion depth limit has been exceeded!")]
fn read_zerosized() {
    struct ZeroSized {}

    static mut REC_DEPTH: u8 = 100;

    impl<'a> Readable<'a> for ZeroSized {
        fn read<'r>(mut s: bytecraft::reader::ReadStream<'a, 'r>) -> Result<Self> {
            if let Some(new) = unsafe { REC_DEPTH.checked_sub(1) } {
                unsafe { REC_DEPTH = new };
            } else {
                panic!("The recursion depth limit has been exceeded!");
            }

            let _: ZeroSized = s.read::<ZeroSized>()?;
            Ok(ZeroSized {})
        }
    }

    let mut reader: ByteReader = ByteReader::new(&[0x01, 0x02]);
    let _: ZeroSized = reader.read().unwrap();
    assert_eq!(reader.position(), 0);
}

#[test]
fn read_custom() -> Result<()> {
    #[derive(Debug, PartialEq, Eq)]
    pub struct LableValue(u32, i8, [u16; 12]);

    impl<'a> Readable<'a> for LableValue {
        fn read<'r>(mut s: bytecraft::reader::ReadStream<'a, 'r>) -> Result<Self> {
            let (f1, f2, f3) = s.read::<(u32, i8, [u16; 12])>()?;
            Ok(Self(f1, f2, f3))
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

    let data: [u8; 45] = [
        0x04, 0x00, 0x00, 0x00, b'B', b'y', b't', b'e', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xF0, 0x12, 0x34, 0x56, 0x78, 0xFB, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05,
        0x00, 0x06, 0x00, 0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x0A, 0x00, 0x0B, 0x00, 0x0C, 0x00,
    ];

    let mut lreader: ByteReader = ByteReader::with_endian(&data, Endian::Little);

    let label: Label = lreader.read()?;

    assert_eq!(
        label,
        Label {
            name: "Byte".into(),
            timestamp: 17293822569102704640,
            value: LableValue(0x78563412, -5, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])
        }
    );

    let mut breader: ByteReader = ByteReader::with_endian(&data, Endian::Big);

    let res: Result<Label> = breader.read();

    assert_eq!(
        matches!(
            res,
            Err(Error::InsufficientData {
                requested: 0x4000000,
                available: 41
            })
        ),
        true
    );

    Ok(())
}

#[test]
fn read_tree() -> Result<()> {
    #[derive(Debug)]
    struct TreeNode {
        val: u32,
        l: Option<Box<TreeNode>>,
        r: Option<Box<TreeNode>>,
    }

    impl PartialEq for TreeNode {
        fn eq(&self, other: &Self) -> bool {
            if self.val != other.val {
                return false;
            }

            self.l == other.l && self.r == other.r
        }
    }

    impl<'a> Readable<'a> for TreeNode {
        fn read<'r>(mut s: bytecraft::reader::ReadStream<'a, 'r>) -> Result<Self> {
            let val: u32 = s.read()?;
            let flg: u8 = s.read()?;

            let l: Option<Box<TreeNode>> = if flg & 0x01 != 0 {
                let left: TreeNode = s.read()?;
                Some(Box::new(left))
            } else {
                None
            };

            let r: Option<Box<TreeNode>> = if flg & 0x02 != 0 {
                let right: TreeNode = s.read()?;
                Some(Box::new(right))
            } else {
                None
            };

            Ok(Self { val, l, r })
        }
    }

    let data: [u8; 5] = [0x01, 0x00, 0x00, 0x00, 0x00];
    let mut reader: ByteReader = ByteReader::new(&data);
    let ts: TreeNode = reader.read()?;

    assert_eq!(
        ts,
        TreeNode {
            val: 1,
            l: None,
            r: None
        }
    );

    let data: [u8; 10] = [0x01, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00];
    let mut reader: ByteReader = ByteReader::new(&data);
    let ts: TreeNode = reader.read()?;

    assert_eq!(
        ts,
        TreeNode {
            val: 1,
            l: Some(Box::new(TreeNode {
                val: 2,
                l: None,
                r: None
            })),
            r: None
        }
    );

    let data: [u8; 15] = [
        0x01, 0x00, 0x00, 0x00, 0x03, 0x02, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
    ];
    let mut reader: ByteReader = ByteReader::new(&data);
    let ts: TreeNode = reader.read()?;

    assert_eq!(
        ts,
        TreeNode {
            val: 1,
            l: Some(Box::new(TreeNode {
                val: 2,
                l: None,
                r: None
            })),
            r: Some(Box::new(TreeNode {
                val: 3,
                l: None,
                r: None
            }))
        }
    );

    Ok(())
}

#[test]
#[should_panic(expected = "The recursion depth limit has been exceeded!")]
fn peek_tree() {
    #[derive(Debug)]
    struct TreeNode {
        val: u32,
        l: Option<Box<TreeNode>>,
        r: Option<Box<TreeNode>>,
    }

    impl PartialEq for TreeNode {
        fn eq(&self, other: &Self) -> bool {
            if self.val != other.val {
                return false;
            }

            self.l == other.l && self.r == other.r
        }
    }

    static mut REC_DEPTH: u8 = 100;

    impl<'a> Peekable<'a> for TreeNode {
        fn peek<'r>(s: bytecraft::reader::PeekStream<'a, 'r>) -> Result<Self> {
            if let Some(new) = unsafe { REC_DEPTH.checked_sub(1) } {
                unsafe { REC_DEPTH = new };
            } else {
                panic!("The recursion depth limit has been exceeded!");
            }

            let val: u32 = s.peek()?;
            let flg: u8 = s.peek()?;

            let l: Option<Box<TreeNode>> = if flg & 0x01 != 0 {
                let left: TreeNode = s.peek()?;
                Some(Box::new(left))
            } else {
                None
            };

            let r: Option<Box<TreeNode>> = if flg & 0x02 != 0 {
                let right: TreeNode = s.peek()?;
                Some(Box::new(right))
            } else {
                None
            };

            Ok(Self { val, l, r })
        }
    }

    let data: [u8; 15] = [
        0x01, 0x00, 0x00, 0x00, 0x03, 0x02, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
    ];
    let reader: ByteReader = ByteReader::new(&data);
    let _ts: TreeNode = reader.peek().unwrap();
}

#[test]
fn lifetimes() -> Result<()> {
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
}
