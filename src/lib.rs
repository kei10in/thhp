use std::str;

#[macro_use]
extern crate error_chain;

mod errors;
mod scanner;

pub use errors::*;
use scanner::Scanner;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status<T> {
    Complete(T),
    Incomplete,
}

impl<T> Status<T> {
    pub fn unwrap(self) -> T {
        match self {
            Complete(val) => val,
            Incomplete => panic!("called `Status::unwrap()` on a `Incomplete` value"),
        }
    }

    pub fn is_complete(&self) -> bool {
        match *self {
            Complete(_) => true,
            Incomplete => false,
        }
    }

    pub fn is_incomplete(&self) -> bool {
        match *self {
            Complete(_) => false,
            Incomplete => true,
        }
    }
}

macro_rules! complete {
    ($expr:expr) => (match $expr {
        $crate::Status::Complete(val) => val,
        $crate::Status::Incomplete => {
            return Ok($crate::Status::Incomplete)
        }
    })
}

pub use Status::*;

pub struct Request<'buffer, 'header>
where
    'buffer: 'header,
{
    pub method: &'buffer [u8],
    pub target: &'buffer [u8],
    pub version: &'buffer [u8],
    pub headers: &'header Vec<HeaderField<'buffer>>,
}

impl<'buffer, 'header> Request<'buffer, 'header> {
    pub fn parse(
        buf: &'buffer [u8],
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Status<Request<'buffer, 'header>>> {
        let mut parser = HttpPartParser::new(buf);
        return parser.parse_request(headers);
    }
}

pub struct Response<'buffer, 'header>
where
    'buffer: 'header,
{
    pub version: &'buffer [u8],
    pub status: u16,
    pub reason: &'buffer [u8],
    pub headers: &'header Vec<HeaderField<'buffer>>,
}

impl<'buffer, 'header> Response<'buffer, 'header> {
    pub fn parse(
        buf: &'buffer [u8],
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Status<Response<'buffer, 'header>>> {
        let mut parser = HttpPartParser::new(buf);
        return parser.parse_response(headers);
    }
}

pub struct HeaderField<'buffer> {
    pub name: &'buffer [u8],
    pub value: &'buffer [u8],
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
    fn parse_request<'header>(
        &mut self,
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Status<Request<'buffer, 'header>>> {
        let method = complete!(self.parse_method()?);
        self.consume_space().ok_or(InvalidMethod)?;
        let target = complete!(self.parse_target()?);
        self.consume_space().ok_or(InvalidPath)?;
        let version = complete!(self.parse_http_version()?);
        complete!(self.consume_eol().unwrap_or(Err(InvalidVersion.into()))?);

        complete!(self.parse_headers(headers)?);

        return Ok(Complete(Request::<'buffer, 'header> {
            method: method,
            target: target,
            version: version,
            headers: headers,
        }));
    }

    #[inline]
    fn parse_response<'header>(
        &mut self,
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Status<Response<'buffer, 'header>>> {
        let version = complete!(self.parse_http_version()?);
        if self.eof() {
            return Ok(Incomplete);
        }
        self.consume_space().ok_or(InvalidVersion)?;
        let status = complete!(self.parse_status_code()?);
        self.consume_space().ok_or(InvalidStatusCode)?;
        let reason = complete!(self.parse_reason_phrase()?);
        complete!(self.consume_eol()
            .unwrap_or(Err(InvalidReasonPhrase.into()))?);

        complete!(self.parse_headers(headers)?);

        return Ok(Complete(Response::<'buffer, 'header> {
            version: version,
            status: status,
            reason: reason,
            headers: headers,
        }));
    }

    #[inline]
    fn eof(&mut self) -> bool {
        self.scanner.empty()
    }

    #[inline]
    fn parse_method(&mut self) -> Result<Status<&'buffer [u8]>> {
        match self.scanner.read_while(|x| is_tchar(x)) {
            Some(v) => Ok(Complete(v)),
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_target(&mut self) -> Result<Status<&'buffer [u8]>> {
        match self.scanner.read_while(|x| is_vchar(x)) {
            Some(v) => Ok(Complete(v)),
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_http_version(&mut self) -> Result<Status<&'buffer [u8]>> {
        if let Some(http) = self.scanner.read(5) {
            if http != b"HTTP/" {
                return Err(InvalidVersion.into());
            }
        } else {
            return Ok(Incomplete);
        }

        match self.scanner.read(3) {
            Some(v) => {
                if v.starts_with(b"1.") && is_digit(unsafe { *v.get_unchecked(2) }) {
                    Ok(Complete(v))
                } else {
                    Err(InvalidVersion.into())
                }
            }
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_status_code(&mut self) -> Result<Status<u16>> {
        match self.scanner.read_while(|x| is_digit(x)) {
            Some(v) => if v.len() == 3 {
                unsafe { str::from_utf8_unchecked(v) }
                    .parse::<u16>()
                    .map(|x| Complete(x))
                    .or(Err(InvalidStatusCode.into()))
            } else {
                Err(InvalidStatusCode.into())
            },
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_reason_phrase(&mut self) -> Result<Status<&'buffer [u8]>> {
        match self.scanner.read_while(|x| is_reason_char(x)) {
            Some(v) => Ok(Complete(v)),
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_field_name(&mut self) -> Result<Status<&'buffer [u8]>> {
        match self.scanner.read_while(|x| is_tchar(x)) {
            Some(v) => Ok(Complete(v)),
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn parse_field_value(&mut self) -> Result<Status<&'buffer [u8]>> {
        match self.scanner.read_while(|x| is_field_value_char(x)) {
            Some(v) => Ok(Complete(v)),
            None => Ok(Incomplete),
        }
    }

    #[inline]
    fn consume_space(&mut self) -> Option<usize> {
        self.scanner.skip_if(b" ")
    }

    #[inline]
    fn consume_colon(&mut self) -> Option<usize> {
        self.scanner.skip_if(b":")
    }

    #[inline]
    fn consume_eol(&mut self) -> Option<Result<Status<usize>>> {
        if self.eof() {
            return Some(Ok(Incomplete));
        }

        if self.scanner.skip_if(b"\r").is_some() {
            if self.eof() {
                return Some(Ok(Incomplete));
            } else if self.scanner.skip_if(b"\n").is_some() {
                return Some(Ok(Complete(2)));
            } else {
                return Some(Err(InvalidNewLine.into()));
            }
        } else if self.scanner.skip_if(b"\n").is_some() {
            return Some(Ok(Complete(1)));
        } else {
            return None;
        }
    }

    #[inline]
    fn eol(&mut self) -> Option<Result<Status<usize>>> {
        if self.scanner.skip_if(b"\r").is_some() {
            if self.eof() {
                return Some(Ok(Incomplete));
            } else if self.scanner.skip_if(b"\n").is_some() {
                return Some(Ok(Complete(2)));
            } else {
                return Some(Err(InvalidNewLine.into()));
            }
        } else if self.scanner.skip_if(b"\n").is_some() {
            Some(Ok(Complete(1)))
        } else {
            None
        }
    }

    fn parse_headers<'header>(
        &mut self,
        result: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Status<()>> {
        loop {
            if let Some(r) = self.eol() {
                return r.map(|x| match x {
                    Complete(_) => Complete(()),
                    Incomplete => Incomplete,
                });
            }

            let name = complete!(self.parse_field_name()?);
            self.consume_colon().ok_or(InvalidFieldName)?;
            let value = complete!(self.parse_field_value()?);

            complete!(self.consume_eol().unwrap_or(Err(InvalidFieldValue.into()))?);

            result.push(HeaderField::<'buffer> {
                name: name,
                value: value,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn http_part_parser_request_test() {
        let mut parser = HttpPartParser::new(b"GET / HTTP/1.1\r\na:b\r\n\r\n");
        let method = parser.parse_method();
        assert_eq!(method.unwrap(), Complete(b"GET".as_ref()));

        assert!(parser.consume_space().is_some());

        let target = parser.parse_target();
        assert_eq!(target.unwrap(), Complete(b"/".as_ref()));

        assert!(parser.consume_space().is_some());

        let version = parser.parse_http_version();
        assert_eq!(version.unwrap(), Complete(b"1.1".as_ref()));

        assert!(parser.consume_eol().is_some());

        let name = parser.parse_field_name();
        assert_eq!(name.unwrap(), Complete(b"a".as_ref()));

        assert!(parser.consume_colon().is_some());

        let value = parser.parse_field_value();
        assert_eq!(value.unwrap(), Complete(b"b".as_ref()));

        assert!(parser.consume_eol().is_some());
        assert!(parser.consume_eol().is_some());
        assert!(parser.eof());
    }

    #[test]
    fn http_part_parser_response_test() {
        let mut parser = HttpPartParser::new(b"HTTP/1.1 200 OK\r\na:b\r\n\r\n");
        let version = parser.parse_http_version();
        assert_eq!(version.unwrap(), Complete(b"1.1".as_ref()));

        assert!(parser.consume_space().is_some());

        let status = parser.parse_status_code();
        assert_eq!(status.unwrap(), Complete(200));

        assert!(parser.consume_space().is_some());

        let reason = parser.parse_reason_phrase();
        assert_eq!(reason.unwrap(), Complete(b"OK".as_ref()));

        assert!(parser.consume_eol().is_some());

        let name = parser.parse_field_name();
        assert_eq!(name.unwrap(), Complete(b"a".as_ref()));

        assert!(parser.consume_colon().is_some());

        let value = parser.parse_field_value();
        assert_eq!(value.unwrap(), Complete(b"b".as_ref()));

        assert!(parser.consume_eol().is_some());
        assert!(parser.consume_eol().is_some());
        assert!(parser.eof());
    }

    #[test]
    fn parse_ok_response() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = Response::parse(b"HTTP/1.1 200 OK\r\nname:value\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        let req = result.unwrap().unwrap();
        assert_eq!(req.version, b"1.1");
        assert_eq!(req.status, 200);
        assert_eq!(req.reason, b"OK");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, b"name");
        assert_eq!(req.headers[0].value, b"value");
    }

    #[test]
    fn parse_get_request() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = Request::parse(b"GET / HTTP/1.1\r\nname:value\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        let req = result.unwrap().unwrap();
        assert_eq!(req.method, b"GET");
        assert_eq!(req.target, b"/");
        assert_eq!(req.version, b"1.1");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, b"name");
        assert_eq!(req.headers[0].value, b"value");
    }

    #[test]
    fn parse_a_header_field() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let mut parser = HttpPartParser::new(b"name:value\r\n\r\n");
        let result = parser.parse_headers(&mut headers);
        assert!(result.is_ok());
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, b"name");
        assert_eq!(headers[0].value, b"value");
    }

    #[test]
    fn parse_2_header_fields() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let mut parser = HttpPartParser::new(b"name1:value1\r\nname2:value2\r\n\r\n");
        let result = parser.parse_headers(&mut headers);
        assert!(result.is_ok());
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].name, b"name1");
        assert_eq!(headers[0].value, b"value1");
        assert_eq!(headers[1].name, b"name2");
        assert_eq!(headers[1].value, b"value2");
    }

}
