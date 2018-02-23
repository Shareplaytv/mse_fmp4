use std::fmt;
use std::io::Read;
use std::str;
use byteorder::{BigEndian, ReadBytesExt};

use {ErrorKind, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoxHeader {
    pub size: u32,
    pub kind: BoxType,
}
impl BoxHeader {
    const SIZE: u32 = 8;

    pub fn read_from<R: Read>(mut reader: R) -> Result<Self> {
        let size = track_io!(reader.read_u32::<BigEndian>())?;
        let kind = track!(BoxType::read_from(&mut reader))?;
        track_assert_ne!(size, 1, ErrorKind::Unsupported);
        track_assert_ne!(size, 0, ErrorKind::Unsupported);
        track_assert!(size >= Self::SIZE, ErrorKind::InvalidInput);

        track_assert_ne!(&kind.0, b"uuid", ErrorKind::Unsupported);

        Ok(BoxHeader { size, kind })
    }

    pub fn data_size(&self) -> u32 {
        self.size - Self::SIZE
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullBoxHeader {
    pub size: u32,
    pub kind: BoxType,
    pub version: u8,
    pub flags: u32, // u24
}
impl FullBoxHeader {
    const SIZE: u32 = BoxHeader::SIZE + 4;

    pub fn read_from<R: Read>(mut reader: R) -> Result<Self> {
        let base = track!(BoxHeader::read_from(&mut reader))?;
        track_assert!(base.size >= Self::SIZE, ErrorKind::InvalidInput);

        let n = track_io!(reader.read_u32::<BigEndian>())?;
        let version = (n >> 24) as u8;
        let flags = n & 0xFF_FFFF;

        Ok(FullBoxHeader {
            size: base.size,
            kind: base.kind,
            version,
            flags,
        })
    }

    pub fn data_size(&self) -> u32 {
        self.size - Self::SIZE
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoxType(pub [u8; 4]);
impl BoxType {
    pub fn read_from<R: Read>(mut reader: R) -> Result<Self> {
        let mut buf = [0; 4];
        track_io!(reader.read_exact(&mut buf[..]))?;
        Ok(BoxType(buf))
    }
}
impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(s) = str::from_utf8(&self.0) {
            write!(f, "BoxType(b{:?})", s)
        } else {
            write!(f, "BoxType({:?})", self.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileTypeBox;
impl FileTypeBox {
    pub const BOX_TYPE: BoxType = BoxType(*b"ftyp");
    pub const CONTAINER: &'static str = "File";
    pub const MANDATORY: bool = true;
    pub const QUANTITY: Quantity = Quantity::ExactlyOne;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quantity {
    ExactlyOne,
}