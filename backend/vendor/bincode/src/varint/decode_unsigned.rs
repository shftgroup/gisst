use core::convert::TryInto;

use super::{SINGLE_BYTE_MAX, U128_BYTE, U16_BYTE, U32_BYTE, U64_BYTE};
use crate::{
    config::Endianness,
    de::read::Reader,
    error::{DecodeError, IntegerType},
};

#[inline(never)]
#[cold]
fn deserialize_varint_cold_u16<R>(read: &mut R, endian: Endianness) -> Result<u16, DecodeError>
where
    R: Reader,
{
    let mut bytes = [0u8; 1];
    read.read(&mut bytes)?;
    match bytes[0] {
        byte @ 0..=SINGLE_BYTE_MAX => Ok(u16::from(byte)),
        U16_BYTE => {
            let mut bytes = [0u8; 2];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u16::from_be_bytes(bytes),
                Endianness::Little => u16::from_le_bytes(bytes),
            })
        }
        U32_BYTE => invalid_varint_discriminant(IntegerType::U16, IntegerType::U32),
        U64_BYTE => invalid_varint_discriminant(IntegerType::U16, IntegerType::U64),
        U128_BYTE => invalid_varint_discriminant(IntegerType::U16, IntegerType::U128),
        _ => invalid_varint_discriminant(IntegerType::U16, IntegerType::Reserved),
    }
}

#[inline(never)]
#[cold]
fn deserialize_varint_cold_u32<R>(read: &mut R, endian: Endianness) -> Result<u32, DecodeError>
where
    R: Reader,
{
    let mut bytes = [0u8; 1];
    read.read(&mut bytes)?;
    match bytes[0] {
        byte @ 0..=SINGLE_BYTE_MAX => Ok(u32::from(byte)),
        U16_BYTE => {
            let mut bytes = [0u8; 2];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u32::from(u16::from_be_bytes(bytes)),
                Endianness::Little => u32::from(u16::from_le_bytes(bytes)),
            })
        }
        U32_BYTE => {
            let mut bytes = [0u8; 4];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u32::from_be_bytes(bytes),
                Endianness::Little => u32::from_le_bytes(bytes),
            })
        }
        U64_BYTE => invalid_varint_discriminant(IntegerType::U32, IntegerType::U64),
        U128_BYTE => invalid_varint_discriminant(IntegerType::U32, IntegerType::U128),
        _ => invalid_varint_discriminant(IntegerType::U32, IntegerType::Reserved),
    }
}

#[inline(never)]
#[cold]
fn deserialize_varint_cold_u64<R>(read: &mut R, endian: Endianness) -> Result<u64, DecodeError>
where
    R: Reader,
{
    let mut bytes = [0u8; 1];
    read.read(&mut bytes)?;
    match bytes[0] {
        byte @ 0..=SINGLE_BYTE_MAX => Ok(u64::from(byte)),
        U16_BYTE => {
            let mut bytes = [0u8; 2];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u64::from(u16::from_be_bytes(bytes)),
                Endianness::Little => u64::from(u16::from_le_bytes(bytes)),
            })
        }
        U32_BYTE => {
            let mut bytes = [0u8; 4];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u64::from(u32::from_be_bytes(bytes)),
                Endianness::Little => u64::from(u32::from_le_bytes(bytes)),
            })
        }
        U64_BYTE => {
            let mut bytes = [0u8; 8];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u64::from_be_bytes(bytes),
                Endianness::Little => u64::from_le_bytes(bytes),
            })
        }
        U128_BYTE => invalid_varint_discriminant(IntegerType::U64, IntegerType::U128),
        _ => invalid_varint_discriminant(IntegerType::U64, IntegerType::Reserved),
    }
}

#[inline(never)]
#[cold]
fn deserialize_varint_cold_usize<R>(read: &mut R, endian: Endianness) -> Result<usize, DecodeError>
where
    R: Reader,
{
    let mut bytes = [0u8; 1];
    read.read(&mut bytes)?;
    match bytes[0] {
        byte @ 0..=SINGLE_BYTE_MAX => Ok(byte as usize),
        U16_BYTE => {
            let mut bytes = [0u8; 2];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u16::from_be_bytes(bytes) as usize,
                Endianness::Little => u16::from_le_bytes(bytes) as usize,
            })
        }
        U32_BYTE => {
            let mut bytes = [0u8; 4];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u32::from_be_bytes(bytes) as usize,
                Endianness::Little => u32::from_le_bytes(bytes) as usize,
            })
        }
        U64_BYTE => {
            let mut bytes = [0u8; 8];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u64::from_be_bytes(bytes) as usize,
                Endianness::Little => u64::from_le_bytes(bytes) as usize,
            })
        }
        U128_BYTE => invalid_varint_discriminant(IntegerType::Usize, IntegerType::U128),
        _ => invalid_varint_discriminant(IntegerType::Usize, IntegerType::Reserved),
    }
}

#[inline(never)]
#[cold]
fn deserialize_varint_cold_u128<R>(read: &mut R, endian: Endianness) -> Result<u128, DecodeError>
where
    R: Reader,
{
    let mut bytes = [0u8; 1];
    read.read(&mut bytes)?;
    match bytes[0] {
        byte @ 0..=SINGLE_BYTE_MAX => Ok(u128::from(byte)),
        U16_BYTE => {
            let mut bytes = [0u8; 2];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u128::from(u16::from_be_bytes(bytes)),
                Endianness::Little => u128::from(u16::from_le_bytes(bytes)),
            })
        }
        U32_BYTE => {
            let mut bytes = [0u8; 4];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u128::from(u32::from_be_bytes(bytes)),
                Endianness::Little => u128::from(u32::from_le_bytes(bytes)),
            })
        }
        U64_BYTE => {
            let mut bytes = [0u8; 8];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u128::from(u64::from_be_bytes(bytes)),
                Endianness::Little => u128::from(u64::from_le_bytes(bytes)),
            })
        }
        U128_BYTE => {
            let mut bytes = [0u8; 16];
            read.read(&mut bytes)?;
            Ok(match endian {
                Endianness::Big => u128::from_be_bytes(bytes),
                Endianness::Little => u128::from_le_bytes(bytes),
            })
        }
        _ => invalid_varint_discriminant(IntegerType::U128, IntegerType::Reserved),
    }
}

#[inline(never)]
#[cold]
const fn invalid_varint_discriminant<T>(
    expected: IntegerType,
    found: IntegerType,
) -> Result<T, DecodeError> {
    Err(DecodeError::InvalidIntegerType { expected, found })
}

pub fn varint_decode_u16<R: Reader>(read: &mut R, endian: Endianness) -> Result<u16, DecodeError> {
    if let Some(bytes) = read.peek_read(3) {
        let (discriminant, bytes) = bytes.split_at(1);
        let (out, used) = match discriminant[0] {
            byte @ 0..=SINGLE_BYTE_MAX => (u16::from(byte), 1),
            U16_BYTE => {
                let val = match endian {
                    Endianness::Big => u16::from_be_bytes(bytes[..2].try_into().unwrap()),
                    Endianness::Little => u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                };

                (val, 3)
            }
            U32_BYTE => return invalid_varint_discriminant(IntegerType::U16, IntegerType::U32),
            U64_BYTE => return invalid_varint_discriminant(IntegerType::U16, IntegerType::U64),
            U128_BYTE => return invalid_varint_discriminant(IntegerType::U16, IntegerType::U128),
            _ => return invalid_varint_discriminant(IntegerType::U16, IntegerType::Reserved),
        };

        read.consume(used);
        Ok(out)
    } else {
        deserialize_varint_cold_u16(read, endian)
    }
}

pub fn varint_decode_u32<R: Reader>(read: &mut R, endian: Endianness) -> Result<u32, DecodeError> {
    if let Some(bytes) = read.peek_read(5) {
        let (discriminant, bytes) = bytes.split_at(1);
        let (out, used) = match discriminant[0] {
            byte @ 0..=SINGLE_BYTE_MAX => (u32::from(byte), 1),
            U16_BYTE => {
                let val = match endian {
                    Endianness::Big => u16::from_be_bytes(bytes[..2].try_into().unwrap()),
                    Endianness::Little => u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                };

                (u32::from(val), 3)
            }
            U32_BYTE => {
                let val = match endian {
                    Endianness::Big => u32::from_be_bytes(bytes[..4].try_into().unwrap()),
                    Endianness::Little => u32::from_le_bytes(bytes[..4].try_into().unwrap()),
                };

                (val, 5)
            }
            U64_BYTE => return invalid_varint_discriminant(IntegerType::U32, IntegerType::U64),
            U128_BYTE => return invalid_varint_discriminant(IntegerType::U32, IntegerType::U128),
            _ => return invalid_varint_discriminant(IntegerType::U32, IntegerType::Reserved),
        };

        read.consume(used);
        Ok(out)
    } else {
        deserialize_varint_cold_u32(read, endian)
    }
}

pub fn varint_decode_u64<R: Reader>(read: &mut R, endian: Endianness) -> Result<u64, DecodeError> {
    if let Some(bytes) = read.peek_read(9) {
        let (discriminant, bytes) = bytes.split_at(1);
        let (out, used) = match discriminant[0] {
            byte @ 0..=SINGLE_BYTE_MAX => (u64::from(byte), 1),
            U16_BYTE => {
                let val = match endian {
                    Endianness::Big => u16::from_be_bytes(bytes[..2].try_into().unwrap()),
                    Endianness::Little => u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                };

                (u64::from(val), 3)
            }
            U32_BYTE => {
                let val = match endian {
                    Endianness::Big => u32::from_be_bytes(bytes[..4].try_into().unwrap()),
                    Endianness::Little => u32::from_le_bytes(bytes[..4].try_into().unwrap()),
                };

                (u64::from(val), 5)
            }
            U64_BYTE => {
                let val = match endian {
                    Endianness::Big => u64::from_be_bytes(bytes[..8].try_into().unwrap()),
                    Endianness::Little => u64::from_le_bytes(bytes[..8].try_into().unwrap()),
                };

                (val, 9)
            }
            U128_BYTE => return invalid_varint_discriminant(IntegerType::U32, IntegerType::U128),
            _ => return invalid_varint_discriminant(IntegerType::U32, IntegerType::Reserved),
        };

        read.consume(used);
        Ok(out)
    } else {
        deserialize_varint_cold_u64(read, endian)
    }
}

pub fn varint_decode_usize<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<usize, DecodeError> {
    if let Some(bytes) = read.peek_read(9) {
        let (discriminant, bytes) = bytes.split_at(1);
        let (out, used) = match discriminant[0] {
            byte @ 0..=SINGLE_BYTE_MAX => (byte as usize, 1),
            U16_BYTE => {
                let val = match endian {
                    Endianness::Big => u16::from_be_bytes(bytes[..2].try_into().unwrap()),
                    Endianness::Little => u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                };

                (val as usize, 3)
            }
            U32_BYTE => {
                let val = match endian {
                    Endianness::Big => u32::from_be_bytes(bytes[..4].try_into().unwrap()),
                    Endianness::Little => u32::from_le_bytes(bytes[..4].try_into().unwrap()),
                };

                (val as usize, 5)
            }
            U64_BYTE => {
                let val = match endian {
                    Endianness::Big => u64::from_be_bytes(bytes[..8].try_into().unwrap()),
                    Endianness::Little => u64::from_le_bytes(bytes[..8].try_into().unwrap()),
                };

                (val as usize, 9)
            }
            U128_BYTE => return invalid_varint_discriminant(IntegerType::Usize, IntegerType::U128),
            _ => return invalid_varint_discriminant(IntegerType::Usize, IntegerType::Reserved),
        };

        read.consume(used);
        Ok(out)
    } else {
        deserialize_varint_cold_usize(read, endian)
    }
}

pub fn varint_decode_u128<R: Reader>(
    read: &mut R,
    endian: Endianness,
) -> Result<u128, DecodeError> {
    if let Some(bytes) = read.peek_read(17) {
        let (discriminant, bytes) = bytes.split_at(1);
        let (out, used) = match discriminant[0] {
            byte @ 0..=SINGLE_BYTE_MAX => (u128::from(byte), 1),
            U16_BYTE => {
                let val = match endian {
                    Endianness::Big => u16::from_be_bytes(bytes[..2].try_into().unwrap()),
                    Endianness::Little => u16::from_le_bytes(bytes[..2].try_into().unwrap()),
                };

                (u128::from(val), 3)
            }
            U32_BYTE => {
                let val = match endian {
                    Endianness::Big => u32::from_be_bytes(bytes[..4].try_into().unwrap()),
                    Endianness::Little => u32::from_le_bytes(bytes[..4].try_into().unwrap()),
                };

                (u128::from(val), 5)
            }
            U64_BYTE => {
                let val = match endian {
                    Endianness::Big => u64::from_be_bytes(bytes[..8].try_into().unwrap()),
                    Endianness::Little => u64::from_le_bytes(bytes[..8].try_into().unwrap()),
                };

                (u128::from(val), 9)
            }
            U128_BYTE => {
                let val = match endian {
                    Endianness::Big => u128::from_be_bytes(bytes[..16].try_into().unwrap()),
                    Endianness::Little => u128::from_le_bytes(bytes[..16].try_into().unwrap()),
                };

                (val, 17)
            }
            _ => return invalid_varint_discriminant(IntegerType::Usize, IntegerType::Reserved),
        };

        read.consume(used);
        Ok(out)
    } else {
        deserialize_varint_cold_u128(read, endian)
    }
}
