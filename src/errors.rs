/// An error in parsing the http header.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// Invalid byte in header field name.
    InvalidFieldName,
    /// Invalid byte in header field value.
    InvalidFieldValue,
    /// Invalid byte in newline.
    InvalidNewLine,
    /// Invalid byte in http version.
    InvalidVersion,
    /// Invalid byte in request method.
    InvalidMethod,
    /// Invalid byte in request target.
    InvalidPath,
    /// Invalid byte in status code.
    InvalidStatusCode,
    /// Invalid byte in reason phrase.
    InvalidReasonPhrase,
    /// Too many header fields.
    OutOfCapacity,
}

pub use crate::Error::*;

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

/// A result type in parsing http header.
pub type Result<T> = ::std::result::Result<T, Error>;
