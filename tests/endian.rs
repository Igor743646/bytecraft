use bytecraft::common::Endian;

#[test]
fn formatting() {
    let lendian: Endian = Endian::Little;
    let bendian: Endian = Endian::Big;
    let nendian: Endian = Endian::Native;

    assert_eq!(&format!("{}", lendian), "Endian::Little");
    assert_eq!(&format!("{}", bendian), "Endian::Big");
    assert_eq!(&format!("{}", nendian), "Endian::Native");

    assert_eq!(&format!("{:?}", lendian), "Little");
    assert_eq!(&format!("{:?}", bendian), "Big");
    assert_eq!(&format!("{:?}", nendian), "Native");

    assert_eq!(&format!("{:#?}", lendian), "Little");
    assert_eq!(&format!("{:#?}", bendian), "Big");
    assert_eq!(&format!("{:#?}", nendian), "Native");

    assert_eq!(&format!("{:>5}", lendian), "Endian::Little");
    assert_eq!(&format!("{:>5}", bendian), "Endian::Big");
    assert_eq!(&format!("{:>5}", nendian), "Endian::Native");

    assert_eq!(&format!("{:>15}", lendian), " Endian::Little");
    assert_eq!(&format!("{:>15}", bendian), "    Endian::Big");
    assert_eq!(&format!("{:>15}", nendian), " Endian::Native");

    assert_eq!(&format!("{:<15}", lendian), "Endian::Little ");
    assert_eq!(&format!("{:<15}", bendian), "Endian::Big    ");
    assert_eq!(&format!("{:<15}", nendian), "Endian::Native ");

    assert_eq!(&format!("{:->15}", lendian), "-Endian::Little");
    assert_eq!(&format!("{:->15}", bendian), "----Endian::Big");
    assert_eq!(&format!("{:->15}", nendian), "-Endian::Native");
}
