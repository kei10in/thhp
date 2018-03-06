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

#[inline]
fn position<P>(buf: &[u8], mut predicate: P) -> Result<usize>
where
    P: FnMut(&u8) -> bool,
{
    let mut i = 0;
    loop {
        if let Some(c) = buf.get(i) {
            if predicate(c) {
                return Ok(i);
            }
            i += 1;
        } else {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }
    }
}

impl<'buffer, 'header> Request<'buffer, 'header> {
    pub fn parse(
        buf: &'buffer [u8],
        headers: &'header mut Vec<HeaderField<'buffer>>,
    ) -> Result<Request<'buffer, 'header>> {
        let mut s = 0;
        let mut i = 0;

        i += position(buf, |&x| x == b' ')?;
        let method = &buf.get(s..i).unwrap();

        i += 1;
        s = i;

        i += position(&buf.get(i..).unwrap(), |&x| x == b' ')?;

        let target = &buf.get(s..i).unwrap();

        i += 1;

        if !buf.get(i..).unwrap().starts_with(b"HTTP/") {
            return Err(ErrorKind::InvalidHeaderFormat.into());
        }

        i += 5;
        s = i;

        i += position(&buf.get(i..).unwrap(), |&x| x == b'\r')?;
        let version = &buf.get(s..i).unwrap();

        i += 1;
        i += 1; // '\n'

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
        if buf[i] == b'\r' {
            break;
        }

        s = i;

        i += position(&buf[i..], |&x| x == b':')?;
        let name = s..i;

        i += 1;

        s = i;
        i += position(&buf[i..], |&x| x == b'\r')?;
        let value = s..i;

        i += 1;

        result.push(HeaderField::<'buffer> {
            name: &buf[name],
            value: &buf[value],
        });

        if buf[i] == b'\n' {
            i += 1;
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
