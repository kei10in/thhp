#[macro_use]
extern crate error_chain;

use std::slice::Iter;

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

impl<'buffer, 'header> Request<'buffer, 'header> {
    pub fn parse(
        buf: &'buffer [u8],
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Request<'buffer, 'header>> {
        Request::parse_impl(&mut buf.iter(), headers)
    }

    fn parse_impl(
        it: &mut Iter<'buffer, u8>,
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Request<'buffer, 'header>> {
        let mut s = it.as_slice();
        let mut p = it.position(|x| *x == b' ');
        if p.is_none() {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }
        let method = &s[0..p.unwrap()];

        s = it.as_slice();
        p = it.position(|x| *x == b' ');
        if p.is_none() {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }
        let target = &s[0..p.unwrap()];

        if !it.as_slice().starts_with(b"HTTP/") {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }

        it.nth(4);

        s = it.as_slice();
        p = it.position(|x| *x == b'\r');
        if p.is_none() {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }
        let version = &s[0..p.unwrap()];

        it.next(); // '\n'

        parse_headers_impl(it, headers)?;

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
    return parse_headers_impl(&mut buf.iter(), result);
}

pub fn parse_headers_impl<'buffer, 'header>(
    it: &mut Iter<'buffer, u8>,
    result: &'header mut Vec<HeaderField<'buffer>>,
) -> Result<()> {
    if it.len() == 0 {
        return Err(ErrorKind::Incomplete.into());
    }

    let mut s;
    loop {
        s = it.as_slice();
        if s[0] == b'\r' {
            break;
        }

        let mut p = it.position(|x| *x == b':');
        if p.is_none() {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }
        let name = &s[0..p.unwrap()];

        s = it.as_slice();
        p = it.position(|x| *x == b'\r');
        if p.is_none() {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }
        let value = &s[0..p.unwrap()];

        result.push(HeaderField::<'buffer> {
            name: name,
            value: value,
        });

        s = it.as_slice();
        if s[0] == b'\n' {
            it.next();
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
}
