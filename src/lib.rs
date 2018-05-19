#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#![cfg_attr(feature = "nightly", feature(stdsimd))]
//! # thhp
//!
//! A parser library for HTTP/1.x requests and responses.
#[cfg(not(feature = "std"))]
extern crate core as std;

use std::ops;
use std::str;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "arrayvec")]
extern crate arrayvec;

#[cfg(feature = "arrayvec")]
mod arrayvec_header;
mod errors;
mod scanner;
mod simd;
mod vec_header;

pub use errors::*;
use scanner::Scanner;

/// A variants of parsing status.
///
/// `Complete` is used when done and succeeded.
/// `Incomplete` is used when valid but parsing ended prematurely.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status<T> {
    /// Represents parsing is done and succeeded completely.
    Complete(T),
    /// Represents parsing is not done but valid.
    Incomplete,
}

pub use Status::*;

impl<T> Status<T> {
    /// Unwraps a status if `Complete(v)`.
    pub fn unwrap(self) -> T {
        match self {
            Complete(val) => val,
            Incomplete => panic!("called `Status::unwrap()` on a `Incomplete` value"),
        }
    }

    /// Returns `true` if the status is a `Complete`.
    pub fn is_complete(&self) -> bool {
        match *self {
            Complete(_) => true,
            Incomplete => false,
        }
    }

    /// Returns `true` if the status is a `Incomplete`.
    pub fn is_incomplete(&self) -> bool {
        match *self {
            Complete(_) => false,
            Incomplete => true,
        }
    }
}

macro_rules! complete {
    ($expr:expr) => {
        match $expr {
            $crate::Status::Complete(val) => val,
            $crate::Status::Incomplete => return Ok($crate::Status::Incomplete),
        }
    };
}

/// A parsed request.
///
/// ## Example
///
/// ```no_run
/// let buf = b"GET / HTTP/1.1\r\nHost: example.com";
/// let mut headers = Vec::<thhp::HeaderField>::with_capacity(16);
/// match thhp::Request::parse(buf, &mut headers) {
///     Ok(thhp::Complete((ref req, len))) => {
///         // Use request.
///     },
///     Ok(thhp::Incomplete) => {
///         // Read more and parse again.
///     },
///     Err(err) => {
///         // Handle error.
///     }
/// }
/// ```
pub struct Request<'headers, 'buffer: 'headers> {
    /// The request method.
    pub method: &'buffer str,
    /// The request target.
    pub target: &'buffer str,
    /// The http minor version.
    pub minor_version: u8,
    /// The request header fields.
    pub headers: &'headers [HeaderField<'buffer>],
}

impl<'headers, 'buffer: 'headers> Request<'headers, 'buffer> {
    /// Parse the buffer as http request.
    pub fn parse<Headers>(
        buf: &'buffer [u8],
        headers: &'headers mut Headers,
    ) -> Result<Status<(Self, usize)>>
    where
        Headers: HeaderFieldCollection<'buffer>,
    {
        let mut parser = HttpPartParser::new(buf);
        Ok(Complete((
            complete!(parser.parse_request(headers)?),
            buf.len() - parser.len(),
        )))
    }
}

/// A parsed response.
///
/// ## Example
///
/// ```no_run
/// let buf = b"HTTP/1.1 200 OK\r\nHost: example.com";
/// let mut headers = Vec::<thhp::HeaderField>::with_capacity(16);
/// match thhp::Response::parse(buf, &mut headers) {
///     Ok(thhp::Complete((ref res, len))) => {
///         // Use reqest.
///     },
///     Ok(thhp::Incomplete) => {
///         // Read more and parse again.
///     },
///     Err(err) => {
///         // Handle error.
///     }
/// }
/// ```
pub struct Response<'headers, 'buffer: 'headers> {
    /// The http minor version.
    pub minor_version: u8,
    /// The status code.
    pub status: u16,
    /// The reason phrase
    pub reason: &'buffer str,
    /// The response header fields.
    pub headers: &'headers [HeaderField<'buffer>],
}

impl<'headers, 'buffer: 'headers> Response<'headers, 'buffer> {
    /// Parse the buffer as http response.
    pub fn parse<Headers>(
        buf: &'buffer [u8],
        headers: &'headers mut Headers,
    ) -> Result<Status<(Self, usize)>>
    where
        Headers: HeaderFieldCollection<'buffer>,
    {
        let mut parser = HttpPartParser::new(buf);
        Ok(Complete((
            complete!(parser.parse_response(headers)?),
            buf.len() - parser.len(),
        )))
    }
}

/// A parsed header field.
pub struct HeaderField<'buffer> {
    /// The header field name.
    pub name: &'buffer str,
    /// The header field value.
    pub value: &'buffer str,
}

/// Trait for a container of header fields.
pub trait HeaderFieldCollection<'buffer>: ops::Deref<Target = [HeaderField<'buffer>]> {
    fn push(&mut self, header_field: HeaderField<'buffer>) -> Result<()>;
}

macro_rules! make_bool_table {
    ($($v:expr,)*) => ([
        $($v != 0,)*
    ])
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const TCHAR_MAP: [bool; 256] = make_bool_table![
    // Control characters
// \0                   \a \b \t \n \v \f \r
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//                                  \e
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,

    // Visible characters
// SP  !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0,

    // Non ascii characters
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[inline]
fn is_tchar(c: u8) -> bool {
    TCHAR_MAP[c as usize]
}

#[inline]
fn is_vchar(c: u8) -> bool {
    0x20 < c && c < 0x7F
}

#[inline]
fn is_digit(c: u8) -> bool {
    b'0' <= c && c <= b'9'
}

#[inline]
fn to_digit(c: u8) -> Option<u8> {
    if is_digit(c) {
        Some(c - b'0')
    } else {
        None
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const REASON_CHAR_MAP: [bool; 256] = make_bool_table![
    // Control characters
// \0                   \a \b \t \n \v \f \r
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
//                                  \e
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,

    // Visible characters
// SP  !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,

    // Non ascii characters
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[inline]
fn is_reason_char(c: u8) -> bool {
    REASON_CHAR_MAP[c as usize]
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const FIELD_VALUE_CHAR_MAP: [bool; 256] = make_bool_table![
    // Control characters
// \0                   \a \b \t \n \v \f \r
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
//                                  \e
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,

    // Visible characters
// SP  !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,

    // Non ascii characters
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[inline]
fn is_field_value_char(c: u8) -> bool {
    FIELD_VALUE_CHAR_MAP[c as usize]
}

struct HttpPartParser<'buffer> {
    scanner: Scanner<'buffer>,
}

impl<'buffer> HttpPartParser<'buffer> {
    #[inline]
    fn new(buf: &'buffer [u8]) -> HttpPartParser {
        HttpPartParser {
            scanner: Scanner::new(buf),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.scanner.len()
    }

    #[inline]
    fn parse_request<'headers, Headers>(
        &mut self,
        headers: &'headers mut Headers,
    ) -> Result<Status<Request<'headers, 'buffer>>>
    where
        Headers: HeaderFieldCollection<'buffer>,
    {
        complete!(self.skip_empty_lines()?);
        Ok(Complete(Request::<'headers, 'buffer> {
            method: complete!(self.parse_request_method()?),
            target: complete!(self.parse_request_target()?),
            minor_version: complete!(self.parse_request_http_version()?),
            headers: complete!(self.parse_headers(headers)?),
        }))
    }

    #[inline]
    fn parse_response<'headers, Headers>(
        &mut self,
        headers: &'headers mut Headers,
    ) -> Result<Status<Response<'headers, 'buffer>>>
    where
        Headers: HeaderFieldCollection<'buffer>,
    {
        complete!(self.skip_empty_lines()?);
        Ok(Complete(Response::<'headers, 'buffer> {
            minor_version: complete!(self.parse_response_http_version()?),
            status: complete!(self.parse_response_status_code()?),
            reason: complete!(self.parse_response_reason_phrase()?),
            headers: complete!(self.parse_headers(headers)?),
        }))
    }

    #[inline]
    fn eof(&mut self) -> bool {
        self.scanner.empty()
    }

    #[inline]
    fn parse_request_method(&mut self) -> Result<Status<&'buffer str>> {
        match self.scanner.read_while(|x| is_tchar(x)) {
            Some(v) => {
                if self.consume_space() {
                    Ok(Complete(unsafe { str::from_utf8_unchecked(v) }))
                } else {
                    Err(InvalidMethod)
                }
            }
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_request_target(&mut self) -> Result<Status<&'buffer str>> {
        match self.scanner.read_while(|x| is_vchar(x)) {
            Some(v) => {
                if self.consume_space() {
                    Ok(Complete(unsafe { str::from_utf8_unchecked(v) }))
                } else {
                    Err(InvalidPath)
                }
            }
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_request_http_version(&mut self) -> Result<Status<u8>> {
        match self.parse_http_version() {
            Ok(Complete(v)) => {
                if complete!(self.consume_eol()?) {
                    Ok(Complete(v))
                } else {
                    Err(InvalidVersion)
                }
            }
            v => v,
        }
    }

    #[inline]
    fn parse_response_http_version(&mut self) -> Result<Status<u8>> {
        match self.parse_http_version() {
            Ok(Complete(v)) => {
                if self.eof() {
                    Ok(Incomplete)
                } else {
                    if self.consume_space() {
                        Ok(Complete(v))
                    } else {
                        Err(InvalidVersion)
                    }
                }
            }
            v => v,
        }
    }

    #[inline]
    fn parse_response_status_code(&mut self) -> Result<Status<u16>> {
        match self.scanner.read_while(|x| is_digit(x)) {
            Some(v) => if v.len() == 3 {
                if self.consume_space() {
                    unsafe { str::from_utf8_unchecked(v) }
                        .parse::<u16>()
                        .map(|x| Complete(x))
                        .or(Err(InvalidStatusCode))
                } else {
                    Err(InvalidStatusCode)
                }
            } else {
                Err(InvalidStatusCode)
            },
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_response_reason_phrase(&mut self) -> Result<Status<&'buffer str>> {
        match self.scanner.read_while(|x| is_reason_char(x)) {
            Some(v) => {
                if complete!(self.consume_eol()?) {
                    Ok(Complete(unsafe { str::from_utf8_unchecked(v) }))
                } else {
                    Err(InvalidReasonPhrase)
                }
            }
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_http_version(&mut self) -> Result<Status<u8>> {
        if let Some(http) = self.scanner.read(8) {
            unsafe {
                if *http.get_unchecked(0) == b'H' && *http.get_unchecked(1) == b'T'
                    && *http.get_unchecked(2) == b'T'
                    && *http.get_unchecked(3) == b'P'
                    && *http.get_unchecked(4) == b'/'
                    && *http.get_unchecked(5) == b'1'
                    && *http.get_unchecked(6) == b'.'
                {
                    let c = http.get_unchecked(7);
                    match to_digit(*c) {
                        Some(v) => Ok(Complete(v)),
                        None => Err(InvalidVersion),
                    }
                } else {
                    Err(InvalidVersion)
                }
            }
        } else if self.scanner.empty() {
            Ok(Incomplete)
        } else if self.scanner.is_head_of(b"HTTP/1.") {
            Ok(Incomplete)
        } else {
            Err(InvalidVersion)
        }
    }

    #[inline]
    fn parse_headers<'headers, Headers>(
        &mut self,
        result: &'headers mut Headers,
    ) -> Result<Status<(&'headers [HeaderField<'buffer>])>>
    where
        Headers: HeaderFieldCollection<'buffer>,
    {
        loop {
            if complete!(self.skip_eol()?) {
                break;
            }

            let header = complete!(self.parse_header_field()?);
            result.push(header)?;
        }

        Ok(Complete(result))
    }

    #[inline]
    fn parse_header_field(&mut self) -> Result<Status<HeaderField<'buffer>>> {
        Ok(Complete(HeaderField::<'buffer> {
            name: complete!(self.parse_field_name()?),
            value: complete!(self.parse_field_value()?),
        }))
    }

    #[inline]
    fn parse_field_name(&mut self) -> Result<Status<&'buffer str>> {
        match self.scanner.read_while(|x| is_tchar(x)) {
            Some(v) => {
                if self.consume_name_value_separator() {
                    Ok(Complete(unsafe { str::from_utf8_unchecked(v) }))
                } else {
                    Err(InvalidFieldName)
                }
            }
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_field_value(&mut self) -> Result<Status<&'buffer str>> {
        match self.read_field_value() {
            Some(v) => {
                if complete!(self.consume_eol()?) {
                    Ok(Complete(unsafe { str::from_utf8_unchecked(v) }))
                } else {
                    Err(InvalidFieldValue)
                }
            }
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn read_field_value(&mut self) -> Option<&'buffer [u8]> {
        #[cfg(feature = "nightly")]
        {
            if is_x86_feature_detected!("sse4.2") {
                let range = b"\x00\x08\x0A\x1F\x7F\xFF".into();
                return self.scanner
                    .read_while_fast(&range, |x| is_field_value_char(x));
            } else {
                return self.scanner.read_while(|x| is_field_value_char(x));
            }
        }

        #[cfg(not(feature = "nightly"))]
        return self.scanner.read_while(|x| is_field_value_char(x));
    }

    #[inline]
    fn consume_space(&mut self) -> bool {
        self.scanner.skip_if(b" ")
    }

    #[inline]
    fn consume_name_value_separator(&mut self) -> bool {
        if self.consume_colon() {
            self.consume_optional_whitespace();
            true
        } else {
            false
        }
    }

    #[inline]
    fn consume_colon(&mut self) -> bool {
        self.scanner.skip_if(b":")
    }

    #[inline]
    fn consume_optional_whitespace(&mut self) -> usize {
        self.scanner
            .read_while(|x| x == b' ' || x == b'\t')
            .map_or(0, |x| x.len())
    }

    #[inline]
    fn consume_eol(&mut self) -> Result<Status<bool>> {
        if self.scanner.skip_if(b"\r\n") {
            Ok(Complete(true))
        } else if self.scanner.skip_if(b"\n") {
            Ok(Complete(true))
        } else if self.scanner.skip_if(b"\r") {
            if self.eof() {
                Ok(Incomplete)
            } else {
                Err(InvalidNewLine)
            }
        } else {
            if self.eof() {
                Ok(Incomplete)
            } else {
                Ok(Complete(false))
            }
        }
    }

    #[inline]
    fn skip_eol(&mut self) -> Result<Status<bool>> {
        match self.scanner.peek(0) {
            Some(&b'\r') => match self.scanner.peek(1) {
                Some(&b'\n') => {
                    unsafe { self.scanner.skip_unchecked(2) };
                    Ok(Complete(true))
                }
                Some(_) => Err(InvalidNewLine),
                None => Ok(Incomplete),
            },
            Some(&b'\n') => {
                unsafe { self.scanner.skip_unchecked(1) };
                Ok(Complete(true))
            }
            Some(_) => Ok(Complete(false)),
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn skip_empty_lines(&mut self) -> Result<Status<()>> {
        loop {
            if !complete!(self.skip_eol()?) {
                return Ok(Complete(()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::vec::Vec;

    use *;

    #[test]
    fn http_part_parser_request_test() {
        let mut parser = HttpPartParser::new(b"GET / HTTP/1.1\r\na:b\r\n\r\n");
        let method = parser.parse_request_method();
        assert_eq!(method.unwrap(), Complete("GET"));

        let target = parser.parse_request_target();
        assert_eq!(target.unwrap(), Complete("/"));

        let version = parser.parse_request_http_version();
        assert_eq!(version.unwrap(), Complete(1));

        let name = parser.parse_field_name();
        assert_eq!(name.unwrap(), Complete("a"));

        let value = parser.parse_field_value();
        assert_eq!(value.unwrap(), Complete("b"));

        assert!(parser.consume_eol().is_ok());
        assert!(parser.eof());
    }

    #[test]
    fn http_part_parser_response_test() {
        let mut parser = HttpPartParser::new(b"HTTP/1.1 200 OK\r\na:b\r\n\r\n");
        let version = parser.parse_response_http_version();
        assert_eq!(version.unwrap(), Complete(1));

        let status = parser.parse_response_status_code();
        assert_eq!(status.unwrap(), Complete(200));

        let reason = parser.parse_response_reason_phrase();
        assert_eq!(reason.unwrap(), Complete("OK"));

        let name = parser.parse_field_name();
        assert_eq!(name.unwrap(), Complete("a"));

        let value = parser.parse_field_value();
        assert_eq!(value.unwrap(), Complete("b"));

        assert!(parser.consume_eol().is_ok());
        assert!(parser.eof());
    }

    #[test]
    fn parse_a_header_field() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let mut parser = HttpPartParser::new(b"name:value\r\n\r\n");

        let r = parser.parse_headers(&mut headers);
        assert!(r.is_ok());

        let s = r.unwrap();
        assert!(s.is_complete());

        let hs = s.unwrap();
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].name, "name");
        assert_eq!(hs[0].value, "value");
    }

    #[test]
    fn parse_2_header_fields() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let mut parser = HttpPartParser::new(b"name1:value1\r\nname2:value2\r\n\r\n");

        let r = parser.parse_headers(&mut headers);
        assert!(r.is_ok());

        let s = r.unwrap();
        assert!(s.is_complete());

        let hs = s.unwrap();
        assert_eq!(hs.len(), 2);
        assert_eq!(hs[0].name, "name1");
        assert_eq!(hs[0].value, "value1");
        assert_eq!(hs[1].name, "name2");
        assert_eq!(hs[1].value, "value2");
    }
}
