pub struct Request<'buffer> {
    pub method: &'buffer [u8],
    pub target: &'buffer [u8],
    pub version: &'buffer [u8],
}

pub fn parse_request<'buffer>(buf: &'buffer [u8]) -> Option<Request<'buffer>> {
    if buf.is_empty() {
        return None;
    }

    let mut s = 0;
    let mut i = 0;

    while buf[i] != b' ' {
        i += 1;
    }
    let method_index = s..i;

    i += 1;
    s = i;

    while buf[i] != b' ' {
        i += 1;
    }
    let target_index = s..i;

    i += 1;
    s = i;

    if &buf[s..s + 5] != b"HTTP/" {
        return None;
    }

    s += 5;
    i = s;
    while buf[i] != b'\r' {
        i += 1;
    }
    let version_index = s..i;

    return Some(Request::<'buffer> {
        method: &buf[method_index],
        target: &buf[target_index],
        version: &buf[version_index],
    });
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
}
