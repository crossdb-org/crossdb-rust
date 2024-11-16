use crate::*;
use cidr::{IpInet, Ipv4Inet, Ipv6Inet};
use mac_address::MacAddress;
use std::ffi::c_char;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::read;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Null,
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Timestamp(i64),
    String(&'a str),
    Binary(&'a [u8]),
    Bool(bool),
    Inet(IpInet),
    Mac(MacAddress),
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
            Value::Timestamp(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Binary(v) => write!(f, "{:?}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Inet(v) => write!(f, "{:?}", v),
            Value::Mac(v) => write!(f, "{:?}", v),
        }
    }
}

impl<'a> Value<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *const c_void, t: DataType) -> Self {
        if ptr.is_null() {
            return Self::Null;
        }
        match t {
            DataType::Null => Self::Null,
            DataType::TinyInt => Self::I8(read(ptr as *const i8)),
            DataType::UTinyInt => todo!(),
            DataType::SmallInt => Self::I16(read(ptr as *const i16)),
            DataType::USmallInt => todo!(),
            DataType::Int => Self::I32(read(ptr as *const i32)),
            DataType::UInt => todo!(),
            DataType::BigInt => Self::I64(read(ptr as *const i64)),
            DataType::UBigInt => todo!(),
            DataType::Float => Self::F32(read(ptr as *const f32)),
            DataType::Double => Self::F64(read(ptr as *const f64)),
            DataType::Timestamp => Self::Timestamp(read(ptr as *const i64)),
            DataType::Char | DataType::VChar => {
                let str = CStr::from_ptr(ptr as *const c_char).to_str().unwrap();
                Self::String(str)
            }
            DataType::Binary | DataType::VBinary => {
                let len = read((ptr as *const u8).offset(-2) as *const u16);
                let data = from_raw_parts(ptr as *const u8, len as usize);
                Self::Binary(data)
            }
            DataType::Bool => Self::Bool(*(ptr as *const i8) == 1),
            DataType::Inet => {
                let bytes = from_raw_parts(ptr as *const u8, 18);
                let mask = bytes[0];
                let family = bytes[1];
                match family {
                    4 => {
                        let mut buf = [0; 4];
                        buf.copy_from_slice(&bytes[2..6]);
                        let net = Ipv4Inet::new(Ipv4Addr::from(buf), mask).unwrap();
                        Self::Inet(IpInet::V4(net))
                    }
                    6 => {
                        let mut buf = [0; 16];
                        buf.copy_from_slice(&bytes[2..18]);
                        let net = Ipv6Inet::new(Ipv6Addr::from(buf), mask).unwrap();
                        Self::Inet(IpInet::V6(net))
                    }
                    _ => unreachable!(),
                }
            }
            DataType::Mac => {
                let bytes = from_raw_parts(ptr as *const u8, 6);
                let address =
                    MacAddress::new([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]);
                Self::Mac(address)
            }
            DataType::Max => todo!(),
        }
    }
}
