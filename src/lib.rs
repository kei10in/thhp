use std::slice::Iter;

pub struct Request<'buffer> {
    pub method: &'buffer [u8],
    pub target: &'buffer [u8],
    pub version: &'buffer [u8],
}

pub struct HeaderField<'buffer> {
    pub name: &'buffer [u8],
    pub value: &'buffer [u8],
}

pub fn parse_request<'buffer>(buf: &'buffer [u8]) -> Option<Request<'buffer>> {
    parse_request_impl(&mut buf.iter())
}

pub fn parse_request_impl<'buffer>(it: &mut Iter<'buffer, u8>) -> Option<Request<'buffer>> {
    let mut s = it.as_slice();
    let mut p = it.position(|x| *x == b' ');
    if p.is_none() {
        return None;
    }
    let method = &s[0..p.unwrap()];

    s = it.as_slice();
    p = it.position(|x| *x == b' ');
    if p.is_none() {
        return None;
    }
    let target = &s[0..p.unwrap()];

    if !it.as_slice().starts_with(b"HTTP/") {
        return None;
    }

    it.nth(4);

    s = it.as_slice();
    p = it.position(|x| *x == b'\r');
    if p.is_none() {
        return None;
    }
    let version = &s[0..p.unwrap()];

    return Some(Request::<'buffer> {
        method: method,
        target: target,
        version: version,
    });
}

pub fn parse_headers<'buffer>(buf: &'buffer [u8]) -> Vec<HeaderField<'buffer>> {
    return parse_headers_impl(&mut buf.iter());
}

pub fn parse_headers_impl<'buffer>(it: &mut Iter<'buffer, u8>) -> Vec<HeaderField<'buffer>> {
    let mut result = Vec::<HeaderField<'buffer>>::new();

    let mut s;
    loop {
        s = it.as_slice();
        if s[0] == b'\r' {
            break;
        }

        let mut p = it.position(|x| *x == b':');
        if p.is_none() {
            return result;
        }
        let name = &s[0..p.unwrap()];

        s = it.as_slice();
        p = it.position(|x| *x == b'\r');
        if p.is_none() {
            return result;
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

    return result;
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn empty_request_is_unparsable() {
        let buf = [0; 0];
        let result = parse_request(&buf);
        assert!(result.is_none());
    }

    #[test]
    fn parse_get_request() {
        let result = parse_request(b"GET / HTTP/1.1\r\n");
        assert!(result.is_some());
        let req = result.unwrap();
        assert_eq!(req.method, b"GET");
        assert_eq!(req.target, b"/");
        assert_eq!(req.version, b"1.1");
    }

    #[test]
    fn parse_post_request() {
        let result = parse_request(b"POST / HTTP/1.1\r\n");
        assert!(result.is_some());
        let req = result.unwrap();
        assert_eq!(req.method, b"POST");
        assert_eq!(req.target, b"/");
        assert_eq!(req.version, b"1.1");
    }

    #[test]
    fn parse_a_header_field() {
        let result = parse_headers(b"name:value\r\n\r\n");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, b"name");
        assert_eq!(result[0].value, b"value");
    }

    #[test]
    fn parse_2_header_fields() {
        let result = parse_headers(b"name1:value1\r\nname2:value2\r\n\r\n");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, b"name1");
        assert_eq!(result[0].value, b"value1");
        assert_eq!(result[1].name, b"name2");
        assert_eq!(result[1].value, b"value2");
    }
}
