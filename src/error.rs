use crate::Error;
use std::fmt::{self, Debug, Display};

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Message(String),
    UnsupportedI64(i64),
    UnsupportedI128(i128),
    UnsupportedU32(u32),
    UnsupportedU64(u64),
    UnsupportedU128(u128),
    UnsupportedF32(f32),
    UnsupportedF64(f64),
    UnsupportedChar(char),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;
        match &self.kind {
            Message(msg) => formatter.write_str(msg),
            UnsupportedI64(v) => write_unsupported_int(v, formatter),
            UnsupportedI128(v) => write_unsupported_int(v, formatter),
            UnsupportedU32(v) => write_unsupported_int(v, formatter),
            UnsupportedU64(v) => write_unsupported_int(v, formatter),
            UnsupportedU128(v) => write_unsupported_int(v, formatter),
            UnsupportedF32(v) => write_unsupported_float(v, formatter),
            UnsupportedF64(v) => write_unsupported_float(v, formatter),
            UnsupportedChar(v) => write!(
                formatter,
                "serialization of char is not supported: '{}'",
                v.escape_debug(),
            ),
        }
    }
}

fn write_unsupported_int(int: &dyn Display, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(
        formatter,
        "unsupported integer: {}, Starlark only supports up to 32-bit signed integers",
        int,
    )
}

fn write_unsupported_float(float: &dyn Display, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(
        formatter,
        "serialization of floating point is not supported: {}",
        float,
    )
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.kind, formatter)
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(message: T) -> Self {
        Error {
            kind: ErrorKind::Message(message.to_string()),
        }
    }
}

impl serde::ser::StdError for Error {}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

pub(crate) fn unsupported_i64(v: i64) -> Error {
    ErrorKind::UnsupportedI64(v).into()
}

pub(crate) fn unsupported_i128(v: i128) -> Error {
    ErrorKind::UnsupportedI128(v).into()
}

pub(crate) fn unsupported_u32(v: u32) -> Error {
    ErrorKind::UnsupportedU32(v).into()
}

pub(crate) fn unsupported_u64(v: u64) -> Error {
    ErrorKind::UnsupportedU64(v).into()
}

pub(crate) fn unsupported_u128(v: u128) -> Error {
    ErrorKind::UnsupportedU128(v).into()
}

pub(crate) fn unsupported_f32(v: f32) -> Error {
    ErrorKind::UnsupportedF32(v).into()
}

pub(crate) fn unsupported_f64(v: f64) -> Error {
    ErrorKind::UnsupportedF64(v).into()
}

pub(crate) fn unsupported_char(v: char) -> Error {
    ErrorKind::UnsupportedChar(v).into()
}
