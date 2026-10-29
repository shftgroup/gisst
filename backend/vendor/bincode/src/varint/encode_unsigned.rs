use super::{SINGLE_BYTE_MAX, U128_BYTE, U16_BYTE, U32_BYTE, U64_BYTE};
use crate::{config::Endianness, enc::write::Writer, error::EncodeError};

pub fn varint_encode_u16<W: Writer>(
    writer: &mut W,
    endian: Endianness,
    val: u16,
) -> Result<(), EncodeError> {
    if val <= SINGLE_BYTE_MAX.into() {
        writer.write(&[val as u8])
    } else {
        writer.write(&[U16_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&val.to_be_bytes()),
            Endianness::Little => writer.write(&val.to_le_bytes()),
        }
    }
}

pub fn varint_encode_u32<W: Writer>(
    writer: &mut W,
    endian: Endianness,
    val: u32,
) -> Result<(), EncodeError> {
    if val <= SINGLE_BYTE_MAX.into() {
        writer.write(&[val as u8])
    } else if val <= u16::MAX.into() {
        writer.write(&[U16_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&(val as u16).to_be_bytes()),
            Endianness::Little => writer.write(&(val as u16).to_le_bytes()),
        }
    } else {
        writer.write(&[U32_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&val.to_be_bytes()),
            Endianness::Little => writer.write(&val.to_le_bytes()),
        }
    }
}

pub fn varint_encode_u64<W: Writer>(
    writer: &mut W,
    endian: Endianness,
    val: u64,
) -> Result<(), EncodeError> {
    if val <= SINGLE_BYTE_MAX.into() {
        writer.write(&[val as u8])
    } else if val <= u16::MAX.into() {
        writer.write(&[U16_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&(val as u16).to_be_bytes()),
            Endianness::Little => writer.write(&(val as u16).to_le_bytes()),
        }
    } else if val <= u32::MAX.into() {
        writer.write(&[U32_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&(val as u32).to_be_bytes()),
            Endianness::Little => writer.write(&(val as u32).to_le_bytes()),
        }
    } else {
        writer.write(&[U64_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&val.to_be_bytes()),
            Endianness::Little => writer.write(&val.to_le_bytes()),
        }
    }
}

pub fn varint_encode_u128<W: Writer>(
    writer: &mut W,
    endian: Endianness,
    val: u128,
) -> Result<(), EncodeError> {
    if val <= SINGLE_BYTE_MAX.into() {
        writer.write(&[val as u8])
    } else if val <= u16::MAX.into() {
        writer.write(&[U16_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&(val as u16).to_be_bytes()),
            Endianness::Little => writer.write(&(val as u16).to_le_bytes()),
        }
    } else if val <= u32::MAX.into() {
        writer.write(&[U32_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&(val as u32).to_be_bytes()),
            Endianness::Little => writer.write(&(val as u32).to_le_bytes()),
        }
    } else if val <= u64::MAX.into() {
        writer.write(&[U64_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&(val as u64).to_be_bytes()),
            Endianness::Little => writer.write(&(val as u64).to_le_bytes()),
        }
    } else {
        writer.write(&[U128_BYTE])?;
        match endian {
            Endianness::Big => writer.write(&val.to_be_bytes()),
            Endianness::Little => writer.write(&val.to_le_bytes()),
        }
    }
}

pub fn varint_encode_usize<W: Writer>(
    writer: &mut W,
    endian: Endianness,
    val: usize,
) -> Result<(), EncodeError> {
    // usize is being encoded as a u64
    varint_encode_u64(writer, endian, val as u64)
}
