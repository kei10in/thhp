#[derive(Debug, Hash, PartialEq)]
pub enum Error {
    InvalidFieldName,
    InvalidFieldValue,
    InvalidNewLine,
    InvalidVersion,
    InvalidMethod,
    InvalidPath,
    InvalidStatusCode,
    InvalidReasonPhrase,
    OutOfCapacity,
}

pub use Error::*;

impl Error {
    pub fn as_str(&self) -> &'static str {
        match *self {
            InvalidFieldName => "InvalidFieldName",
            InvalidFieldValue => "InvalidFieldValue",
            InvalidNewLine => "InvalidNewLine",
            InvalidVersion => "InvalidVersion",
            InvalidMethod => "InvalidMethod",
            InvalidPath => "InvalidPath",
            InvalidStatusCode => "InvalidStatusCode",
            InvalidReasonPhrase => "InvalidReasonPhrase",
            OutOfCapacity => "OutOfCapacity",
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(fmt, "{:?}", self.as_str())
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
