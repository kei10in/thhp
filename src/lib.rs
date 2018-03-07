#[macro_use]
extern crate error_chain;

mod errors;

pub use errors::*;

pub struct Request<'buffer, 'header>
where
    'buffer: 'header,
{
    pub method: &'buffer [u8],
    pub target: &'buffer [u8],
    pub version: &'buffer [u8],
    pub headers: &'header Vec<HeaderField<'buffer>>,
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

#[inline]
fn read_until<D, A>(buf: &[u8], mut delimitor: D, mut acceptable: A) -> Result<usize>
where
    D: FnMut(&u8) -> bool,
    A: FnMut(&u8) -> bool,
{
    let mut i = 0;
    loop {
        match buf.get(i) {
            Some(c) => {
                if delimitor(c) {
                    return Ok(i);
                }

                if !acceptable(c) {
                    return Err(InvalidHeaderFormat.into());
                }

                i += 1;
            }
            None => return Err(InvalidHeaderFormat.into()),
        }
    }
}

#[inline]
fn parse_method(buf: &[u8]) -> Result<usize> {
    read_until(buf, |&x| x == b' ', |&x| is_tchar(x))
}

#[inline]
fn parse_target(buf: &[u8]) -> Result<usize> {
    read_until(buf, |&x| x == b' ', |&x| is_vchar(x))
}

#[inline]
fn parse_http_version(buf: &[u8]) -> Result<usize> {
    if buf.len() < 3 {
        return Err(Incomplete.into());
    }

    if buf[0] == b'1' && buf[1] == b'.' && is_digit(buf[2]) {
        return Ok(3);
    } else {
        return Err(InvalidHeaderFormat.into());
    }
}

#[inline]
fn parse_field_name(buf: &[u8]) -> Result<usize> {
    read_until(buf, |&x| x == b':', |&x| is_tchar(x))
}

#[inline]
fn parse_field_value(buf: &[u8]) -> Result<usize> {
    read_until(
        buf,
        |&x| x == b'\r' || x == b'\n',
        |&x| is_field_value_char(x),
    )
}

#[inline]
fn parse_eol(buf: &[u8]) -> Option<usize> {
    if buf.get(0) == Some(&b'\r') && buf.get(1) == Some(&b'\n') {
        Some(2)
    } else if buf.get(0) == Some(&b'\n') {
        Some(1)
    } else {
        None
    }
}

impl<'buffer, 'header> Request<'buffer, 'header> {
    pub fn parse(
        buf: &'buffer [u8],
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Request<'buffer, 'header>> {
        let mut s = 0;
        let mut i = 0;

        i += parse_method(buf)?;
        let method = unsafe { buf.get_unchecked(s..i) };

        i += 1;
        s = i;

        i += parse_target(&buf[i..])?;
        let target = unsafe { buf.get_unchecked(s..i) };

        i += 1;

        if !buf.get(i..).unwrap().starts_with(b"HTTP/") {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }

        i += 5;
        s = i;

        i += parse_http_version(&buf[i..])?;
        let version = &buf.get(s..i).unwrap();

        match parse_eol(&buf[i..]) {
            Some(c) => i += c,
            None => return Err(InvalidHeaderFormat.into()),
        }

        parse_headers(&buf[i..], headers)?;

        return Ok(Request::<'buffer, 'header> {
            method: method,
            target: target,
            version: version,
            headers: headers,
        });
    }
}

pub fn parse_headers<'buffer, 'header>(
    buf: &'buffer [u8],
    result: &'header mut Vec<HeaderField<'buffer>>,
) -> Result<()> {
    if buf.len() == 0 {
        return Err(ErrorKind::Incomplete.into());
    }

    let mut s;
    let mut i = 0;
    loop {
        if parse_eol(&buf[i..]).is_some() {
            break;
        }

        s = i;

        i += parse_field_name(buf)?;
        let name = s..i;

        i += 1;

        s = i;
        i += parse_field_value(&buf[i..])?;
        let value = s..i;

        result.push(HeaderField::<'buffer> {
            name: &buf[name],
            value: &buf[value],
        });

        match parse_eol(&buf[i..]) {
            Some(c) => i += c,
            None => return Err(InvalidHeaderFormat.into()),
        }
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn empty_request_is_unparsable() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = Request::parse(b"", &mut headers);
        assert!(result.is_err());
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
        let result = parse_headers(b"name:value\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, b"name");
        assert_eq!(headers[0].value, b"value");
    }

    #[test]
    fn parse_2_header_fields() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        let result = parse_headers(b"name1:value1\r\nname2:value2\r\n\r\n", &mut headers);
        assert!(result.is_ok());
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].name, b"name1");
        assert_eq!(headers[0].value, b"value1");
        assert_eq!(headers[1].name, b"name2");
        assert_eq!(headers[1].value, b"value2");
    }

    #[test]
    fn good() {
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

    fn fail(r: Result<Request>, err: ErrorKind) {
        assert!(r.is_err());
        assert_eq!(*r.err().unwrap().kind(), err);
    }

    #[test]
    fn failures() {
        let mut headers = Vec::<HeaderField>::with_capacity(10);
        fail(
            Request::parse(b"G\x01ET / HTTP/1.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET /a\x01ef HTTP/1.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET / HOGE\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET / HTTP/11.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET / HTTP/A.1\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET / HTTP/1.A\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET / HTTP/1.A\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET / HTTP/1.1\r\na\x01b:xyz\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
        fail(
            Request::parse(b"GET / HTTP/1.1\r\nabc:x\x01z\r\n\r\n", &mut headers),
            InvalidHeaderFormat,
        );
    }
}
