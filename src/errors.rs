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
            InvalidFieldName => "invalid field name",
            InvalidFieldValue => "invalid field value",
            InvalidNewLine => "invalid new line",
            InvalidVersion => "invalid version",
            InvalidMethod => "invalid method",
            InvalidPath => "invalid path",
            InvalidStatusCode => "invalid status code",
            InvalidReasonPhrase => "invalid reason phrase",
            OutOfCapacity => "out of capacity",
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(fmt, "{:?}", self.as_str())
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
