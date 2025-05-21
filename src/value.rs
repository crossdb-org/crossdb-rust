use crate::*;
use cidr::{IpInet, Ipv4Inet, Ipv6Inet};
use mac_address::MacAddress;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Null,
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Timestamp(i64),
    String(&'a str),
    Binary(&'a [u8]),
    Bool(bool),
    Inet(IpInet),
    Mac(MacAddress),
    // TODO: Array
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
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

impl Value<'_> {
    pub(crate) unsafe fn from_ptr(
        res: *mut xdb_res_t,
        row: *mut xdb_row_t,
        i: u16,
        t: DataType,
    ) -> Self {
        if xdb_column_null(res, row, i) {
            return Self::Null;
        }
        match t {
            DataType::Null => Self::Null,
            DataType::TinyInt | DataType::SmallInt | DataType::Int => {
                Self::I32(xdb_column_int(res, row, i))
            }
            DataType::BigInt => Self::I64(xdb_column_int64(res, row, i)),
            DataType::UTinyInt | DataType::USmallInt | DataType::UInt => {
                Self::U32(xdb_column_uint(res, row, i))
            }
            DataType::UBigInt => Self::U64(xdb_column_uint64(res, row, i)),
            DataType::Float => Self::F32(xdb_column_float(res, row, i)),
            DataType::Double => Self::F64(xdb_column_double(res, row, i)),
            DataType::Timestamp => Self::Timestamp(xdb_column_int64(res, row, i)),
            DataType::Char | DataType::VChar => {
                let ptr = xdb_column_str(res, row, i);
                let str = CStr::from_ptr(ptr).to_str().unwrap();
                Self::String(str)
            }
            DataType::Binary | DataType::VBinary => {
                let mut len = 0_i32;
                let ptr = xdb_column_blob(res, row, i, &mut len);
                if len <= 0 {
                    return Self::Null;
                }
                let data = from_raw_parts(ptr as *const u8, len as usize);
                Self::Binary(data)
            }
            DataType::Bool => Self::Bool(xdb_column_bool(res, row, i)),
            DataType::Inet => {
                let inet = *xdb_column_inet(res, row, i);
                match inet.family {
                    4 => {
                        let mut buf = [0; 4];
                        buf.copy_from_slice(&inet.addr[0..4]);
                        let net = Ipv4Inet::new(Ipv4Addr::from(buf), inet.mask).unwrap();
                        Self::Inet(IpInet::V4(net))
                    }
                    6 => {
                        let net = Ipv6Inet::new(Ipv6Addr::from(inet.addr), inet.mask).unwrap();
                        Self::Inet(IpInet::V6(net))
                    }
                    _ => unreachable!(),
                }
            }
            DataType::Mac => {
                let mac = *xdb_column_mac(res, row, i);
                let address = MacAddress::new(mac.addr);
                Self::Mac(address)
            }
            DataType::Json => todo!(),
            DataType::Array => todo!(),
            DataType::Max => todo!(),
        }
    }
}
