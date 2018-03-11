use std::str;

#[macro_use]
extern crate error_chain;

mod errors;
mod scanner;

pub use errors::*;
use scanner::Scanner;

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
    ) -> Result<Request<'buffer, 'header>> {
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
    ) -> Result<Response<'buffer, 'header>> {
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
    ) -> Result<Request<'buffer, 'header>> {
        let method = self.parse_method()?;
        self.consume_space()?;
        let target = self.parse_target()?;
        self.consume_space()?;
        let version = self.parse_http_version()?;
        self.consume_eol()?;

        self.parse_headers(headers)?;

        return Ok(Request::<'buffer, 'header> {
            method: method,
            target: target,
            version: version,
            headers: headers,
        });
    }

    #[inline]
    fn parse_response<'header>(
        &mut self,
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Response<'buffer, 'header>> {
        let version = self.parse_http_version()?;
        self.consume_space()?;
        let status = self.parse_status_code()?;
        self.consume_space()?;
        let reason = self.parse_reason_phrase()?;
        self.consume_eol()?;

        self.parse_headers(headers)?;

        return Ok(Response::<'buffer, 'header> {
            version: version,
            status: status,
            reason: reason,
            headers: headers,
        });
    }

    #[inline]
    #[cfg(test)]
    fn eof(&mut self) -> bool {
        self.scanner.empty()
    }

    #[inline]
    fn parse_method(&mut self) -> Result<&'buffer [u8]> {
        self.scanner
            .read_while(|x| is_tchar(x))
            .ok_or::<Error>(Incomplete.into())
    }

    #[inline]
    fn parse_target(&mut self) -> Result<&'buffer [u8]> {
        self.scanner
            .read_while(|x| is_vchar(x))
            .ok_or::<Error>(Incomplete.into())
    }

    #[inline]
    fn parse_http_version(&mut self) -> Result<&'buffer [u8]> {
        let http = self.scanner.read(5).ok_or::<Error>(Incomplete.into())?;
        if http != b"HTTP/" {
            return Err(InvalidHeaderFormat.into());
        }

        match self.scanner.read(3) {
            Some(v) => {
                if v.starts_with(b"1.") && is_digit(unsafe { *v.get_unchecked(2) }) {
                    Ok(v)
                } else {
                    Err(InvalidHeaderFormat.into())
                }
            }
            None => Err(Incomplete.into()),
        }
    }

    #[inline]
    fn parse_status_code(&mut self) -> Result<u16> {
        match self.scanner.read_while(|x| is_digit(x)) {
            Some(v) => if v.len() == 3 {
                unsafe { str::from_utf8_unchecked(v) }
                    .parse::<u16>()
                    .or(Err(InvalidHeaderFormat.into()))
            } else {
                Err(InvalidHeaderFormat.into())
            },
            None => Err(InvalidHeaderFormat.into()),
        }
    }

    #[inline]
    fn parse_reason_phrase(&mut self) -> Result<&'buffer [u8]> {
        self.scanner
            .read_while(|x| is_reason_char(x))
            .ok_or::<Error>(InvalidHeaderFormat.into())
    }

    #[inline]
    fn parse_field_name(&mut self) -> Result<&'buffer [u8]> {
        self.scanner
            .read_while(|x| is_tchar(x))
            .ok_or::<Error>(Incomplete.into())
    }

    #[inline]
    fn parse_field_value(&mut self) -> Result<&'buffer [u8]> {
        self.scanner
            .read_while(|x| is_field_value_char(x))
            .ok_or(Incomplete.into())
    }

    #[inline]
    fn consume_space(&mut self) -> Result<usize> {
        self.scanner
            .skip_if(b" ")
            .ok_or::<Error>(InvalidHeaderFormat.into())
    }

    #[inline]
    fn consume_colon(&mut self) -> Result<usize> {
        self.scanner
            .skip_if(b":")
            .ok_or::<Error>(InvalidHeaderFormat.into())
    }

    #[inline]
    fn consume_eol(&mut self) -> Result<usize> {
        if self.scanner.empty() {
            return Err(Incomplete.into());
        }

        if self.scanner.skip_if(b"\r\n").is_some() {
            Ok(2)
        } else if self.scanner.skip_if(b"\n").is_some() {
            Ok(1)
        } else {
            Err(InvalidHeaderFormat.into())
        }
    }

    #[inline]
    fn eol(&mut self) -> Option<usize> {
        if self.scanner.skip_if(b"\r\n").is_some() {
            Some(2)
        } else if self.scanner.skip_if(b"\n").is_some() {
            Some(1)
        } else {
            None
        }
    }

    fn parse_headers<'header>(
        &mut self,
        result: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<()> {
        loop {
            if self.eol().is_some() {
                break;
            }

            let name = self.parse_field_name()?;
            self.consume_colon()?;
            let value = self.parse_field_value()?;

            result.push(HeaderField::<'buffer> {
                name: name,
                value: value,
            });

            self.consume_eol()?;
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn http_part_parser_request_test() {
        let mut parser = HttpPartParser::new(b"GET / HTTP/1.1\r\na:b\r\n\r\n");
        let method = parser.parse_method();
        assert_eq!(method.unwrap(), b"GET");

        assert!(parser.consume_space().is_ok());

        let target = parser.parse_target();
        assert_eq!(target.unwrap(), b"/");

        assert!(parser.consume_space().is_ok());

        let version = parser.parse_http_version();
        assert_eq!(version.unwrap(), b"1.1");

        assert!(parser.consume_eol().is_ok());

        let name = parser.parse_field_name();
        assert_eq!(name.unwrap(), b"a");

        assert!(parser.consume_colon().is_ok());

        let value = parser.parse_field_value();
        assert_eq!(value.unwrap(), b"b");

        assert!(parser.consume_eol().is_ok());
        assert!(parser.consume_eol().is_ok());
        assert!(parser.eof());
    }

    #[test]
    fn http_part_parser_response_test() {
        let mut parser = HttpPartParser::new(b"HTTP/1.1 200 OK\r\na:b\r\n\r\n");
        let version = parser.parse_http_version();
        assert_eq!(version.unwrap(), b"1.1");

        assert!(parser.consume_space().is_ok());

        let status = parser.parse_status_code();
        assert_eq!(status.unwrap(), 200);

        assert!(parser.consume_space().is_ok());

        let reason = parser.parse_reason_phrase();
        assert_eq!(reason.unwrap(), b"OK");

        assert!(parser.consume_eol().is_ok());

        let name = parser.parse_field_name();
        assert_eq!(name.unwrap(), b"a");

        assert!(parser.consume_colon().is_ok());

        let value = parser.parse_field_value();
        assert_eq!(value.unwrap(), b"b");

        assert!(parser.consume_eol().is_ok());
        assert!(parser.consume_eol().is_ok());
        assert!(parser.eof());
    }

    #[test]
    fn empty_request_is_unparsable() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = Request::parse(b"", &mut headers);
        assert!(result.is_err());
    }

    #[test]
    fn parse_ok_response() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = Response::parse(b"HTTP/1.1 200 OK\r\nname:value\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        let req = result.unwrap();
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
        let req = result.unwrap();
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

    #[test]
    fn good_request() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        assert!(Request::parse(b"GET / HTTP/1.1\r\n\r\n", &mut headers).is_ok());
        assert!(Request::parse(b"GET / HTTP/1.1\n\n", &mut headers).is_ok());
        assert!(Request::parse(b"GET / HTTP/1.1\r\n\n", &mut headers).is_ok());
        assert!(Request::parse(b"GET / HTTP/1.1\n\r\n", &mut headers).is_ok());
        assert!(Request::parse(b"GET / HTTP/1.1\r\na:b\r\n\r\n", &mut headers).is_ok());
        assert!(Request::parse(b"GET / HTTP/1.1\r\na:b\r\n\n", &mut headers).is_ok());
        assert!(Request::parse(b"GET / HTTP/1.1\r\na:b\n\n", &mut headers).is_ok());
        assert!(Request::parse(b"GET / HTTP/1.1\r\na:b\n\r\n", &mut headers).is_ok());
    }

    macro_rules! fail{
        ($parse: expr, $err: ident) => {
            {
                let r = $parse;
                assert!(r.is_err());
                assert_eq!(*r.err().unwrap().kind(), $err);
            }
        }
    }

    macro_rules! invalid_header_request{
        ($parse: expr) => {
            fail!($parse, InvalidHeaderFormat)
        }
    }

    macro_rules! incomplete{
        ($parse: expr) => {
            fail!($parse, Incomplete)
        }
    }

    #[test]
    fn bad_request() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        invalid_header_request!(Request::parse(b"G\x01ET / HTTP/1.1\r\n\r\n", &mut headers));
        invalid_header_request!(Request::parse(
            b"GET /a\x01ef HTTP/1.1\r\n\r\n",
            &mut headers
        ));
        invalid_header_request!(Request::parse(b"GET / HOGE\r\n\r\n", &mut headers));
        invalid_header_request!(Request::parse(b"GET / HTTP/11.1\r\n\r\n", &mut headers));
        invalid_header_request!(Request::parse(b"GET / HTTP/A.1\r\n\r\n", &mut headers));
        invalid_header_request!(Request::parse(b"GET / HTTP/1.A\r\n\r\n", &mut headers));
        invalid_header_request!(Request::parse(b"GET / HTTP/1.A\r\n\r\n", &mut headers));
        invalid_header_request!(Request::parse(
            b"GET / HTTP/1.1\r\na\x01b:xyz\r\n\r\n",
            &mut headers
        ));
        invalid_header_request!(Request::parse(
            b"GET / HTTP/1.1\r\nabc:x\x01z\r\n\r\n",
            &mut headers
        ));
    }

    #[test]
    fn incomplete_request() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        incomplete!(Request::parse(b"GET", &mut headers));
        incomplete!(Request::parse(b"GET ", &mut headers));
        incomplete!(Request::parse(b"GET /", &mut headers));
        incomplete!(Request::parse(b"GET / ", &mut headers));
        incomplete!(Request::parse(b"GET / HTT", &mut headers));
        incomplete!(Request::parse(b"GET / HTTP/1.", &mut headers));
        incomplete!(Request::parse(b"GET / HTTP/1.1\r\n", &mut headers));
        incomplete!(Request::parse(b"GET / HTTP/1.1\r\na", &mut headers));
        incomplete!(Request::parse(b"GET / HTTP/1.1\r\na:", &mut headers));
        incomplete!(Request::parse(b"GET / HTTP/1.1\r\na:b", &mut headers));
        incomplete!(Request::parse(b"GET / HTTP/1.1\r\na:b\r\n", &mut headers));
    }

}
