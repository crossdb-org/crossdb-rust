use crate::*;

pub enum Value {
    Int(i32),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(CString),
}

trait IntoValue {
    fn into_value(self) -> Result<Value>;
}

macro_rules! impl_value {
    ($t: ty, $v: ident) => {
        impl IntoValue for $t {
            fn into_value(self) -> Result<Value> {
                Ok(Value::$v(self as _))
            }
        }
    };
}
impl_value!(i8, Int);
impl_value!(u8, Int);
impl_value!(i16, Int);
impl_value!(u16, Int);
impl_value!(i32, Int);
impl_value!(u32, Int);
impl_value!(u64, Int64);
impl_value!(i64, Int64);
impl_value!(f32, Float);
impl_value!(f64, Double);

impl IntoValue for String {
    fn into_value(self) -> Result<Value> {
        Ok(Value::String(CString::new(self)?))
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Result<Value> {
        Ok(Value::String(CString::new(self)?))
    }
}

impl IntoValue for CString {
    fn into_value(self) -> Result<Value> {
        Ok(Value::String(self))
    }
}

impl IntoValue for Value {
    fn into_value(self) -> Result<Value> {
        Ok(self)
    }
}

pub enum Params {
    Empty,
    Positional(Vec<Value>),
}

pub trait IntoParams {
    fn into_params(self) -> Result<Params>;
}

impl IntoParams for () {
    fn into_params(self) -> Result<Params> {
        Ok(Params::Empty)
    }
}

impl IntoParams for Params {
    fn into_params(self) -> Result<Params> {
        Ok(self)
    }
}

impl<T: IntoValue> IntoParams for Vec<T> {
    fn into_params(self) -> Result<Params> {
        let mut params = Vec::with_capacity(self.len());
        for param in self {
            params.push(param.into_value()?);
        }
        Ok(Params::Positional(params))
    }
}

impl<T: IntoValue + Clone> IntoParams for &[T] {
    fn into_params(self) -> Result<Params> {
        self.to_vec().into_params()
    }
}

impl<T: IntoValue + Clone, const N: usize> IntoParams for &[T; N] {
    fn into_params(self) -> Result<Params> {
        self.to_vec().into_params()
    }
}

// Copy from:https://github.com/tursodatabase/libsql/blob/main/libsql/src/params.rs#L206-L207
macro_rules! tuple_into_params {
    ($count:literal : $(($field:tt $ftype:ident)),* $(,)?) => {
        impl<$($ftype,)*> IntoParams for ($($ftype,)*) where $($ftype: IntoValue,)* {
            fn into_params(self) -> Result<Params> {
                let params = Params::Positional(vec![$(self.$field.into_value()?),*]);
                Ok(params)
            }
        }
    }
}
tuple_into_params!(1: (0 A));
tuple_into_params!(2: (0 A), (1 B));
tuple_into_params!(3: (0 A), (1 B), (2 C));
tuple_into_params!(4: (0 A), (1 B), (2 C), (3 D));
tuple_into_params!(5: (0 A), (1 B), (2 C), (3 D), (4 E));
tuple_into_params!(6: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F));
tuple_into_params!(7: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G));
tuple_into_params!(8: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H));
tuple_into_params!(9: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I));
tuple_into_params!(10: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J));
tuple_into_params!(11: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K));
tuple_into_params!(12: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L));
tuple_into_params!(13: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M));
tuple_into_params!(14: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M), (13 N));
tuple_into_params!(15: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M), (13 N), (14 O));
tuple_into_params!(16: (0 A), (1 B), (2 C), (3 D), (4 E), (5 F), (6 G), (7 H), (8 I), (9 J), (10 K), (11 L), (12 M), (13 N), (14 O), (15 P));
