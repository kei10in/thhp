extern crate thhp;

use thhp::*;

#[cfg(test)]
mod request {
    use *;

    macro_rules! good {
        ($buf:expr) => {
            good!($buf, |_req| {})
        };
        ($buf:expr, | $req:ident | $body:expr) => {{
            let mut headers = Vec::<HeaderField>::with_capacity(10);
            match Request::parse($buf, &mut headers) {
                Ok(Complete((req, c))) => {
                    assert_eq!(c, $buf.len());
                    closure(req);
                }
                _ => assert!(false),
            }

            fn closure($req: Request) {
                $body
            }
        }};
    }

    macro_rules! fail {
        ($buf:expr, $err:ident) => {{
            let mut headers = Vec::<HeaderField>::with_capacity(1);
            let r = Request::parse($buf, &mut headers);
            assert!(r.is_err());
            assert_eq!(r.err().unwrap(), $err);
        }};
    }

    macro_rules! invalid_method {
        ($parse:expr) => {
            fail!($parse, InvalidMethod)
        };
    }

    macro_rules! invalid_path {
        ($parse:expr) => {
            fail!($parse, InvalidPath)
        };
    }

    macro_rules! invalid_version {
        ($parse:expr) => {
            fail!($parse, InvalidVersion)
        };
    }

    macro_rules! invalid_field_name {
        ($parse:expr) => {
            fail!($parse, InvalidFieldName)
        };
    }

    macro_rules! invalid_field_value {
        ($parse:expr) => {
            fail!($parse, InvalidFieldValue)
        };
    }

    macro_rules! invalid_new_line {
        ($parse:expr) => {
            fail!($parse, InvalidNewLine)
        };
    }

    macro_rules! out_of_capacity {
        ($parse:expr) => {
            fail!($parse, OutOfCapacity)
        };
    }

    macro_rules! incomplete {
        ($buf:expr) => {{
            let mut headers = Vec::<HeaderField>::with_capacity(10);
            let r = Request::parse($buf, &mut headers);
            assert!(r.is_ok());
            assert!(r.unwrap().is_incomplete());
        }};
    }

    #[test]
    fn simple_request() {
        good!(b"GET / HTTP/1.1\r\n\r\n", |req| {
            assert_eq!(req.method, "GET");
            assert_eq!(req.target, "/");
            assert_eq!(req.minor_version, 1);
            assert_eq!(req.headers.len(), 0);
        });
    }

    #[test]
    fn simple_request_with_headers() {
        good!(b"GET / HTTP/1.1\r\na:b\r\nc:d\r\n\r\n", |req| {
            assert_eq!(req.method, "GET");
            assert_eq!(req.target, "/");
            assert_eq!(req.minor_version, 1);
            assert_eq!(req.headers.len(), 2);
            assert_eq!(req.headers[0].name, "a");
            assert_eq!(req.headers[0].value, "b");
            assert_eq!(req.headers[1].name, "c");
            assert_eq!(req.headers[1].value, "d");
        });
    }

    #[test]
    fn accept_various_new_lines() {
        good!(b"GET / HTTP/1.1\n\n");
        good!(b"GET / HTTP/1.1\r\n\n");
        good!(b"GET / HTTP/1.1\n\r\n");
        good!(b"GET / HTTP/1.1\r\na:b\r\n\n");
        good!(b"GET / HTTP/1.1\r\na:b\n\n");
        good!(b"GET / HTTP/1.1\r\na:b\n\r\n");
    }

    #[test]
    fn skip_front_new_lines() {
        good!(b"\r\nGET / HTTP/1.1\r\na:b\n\r\n");
        good!(b"\r\n\r\nGET / HTTP/1.1\r\na:b\n\r\n");
        good!(b"\nGET / HTTP/1.1\r\na:b\n\r\n");
        good!(b"\n\nGET / HTTP/1.1\r\na:b\n\r\n");
    }

    #[test]
    fn bad_request() {
        invalid_method!(b"G\x01ET / HTTP/1.1\r\n\r\n");
        invalid_path!(b"GET /a\x01ef HTTP/1.1\r\n\r\n");
        invalid_version!(b"GET / H\r\n\r\n");
        invalid_version!(b"GET / HOGE\r\n\r\n");
        invalid_version!(b"GET / HTTP/11.1\r\n\r\n");
        invalid_version!(b"GET / HTTP/A.1\r\n\r\n");
        invalid_version!(b"GET / HTTP/1.A\r\n\r\n");
        invalid_version!(b"GET / HTTP/1.A\r\n\r\n");
        invalid_field_name!(b"GET / HTTP/1.1\r\na\x01b:xyz\r\n\r\n");
        invalid_field_value!(b"GET / HTTP/1.1\r\nabc:x\x01z\r\n\r\n");
        invalid_new_line!(b"GET / HTTP/1.1\r\nabc:xyz\ra\n\r\n");
        invalid_new_line!(b"GET / HTTP/1.1\r\nabc:xyz\r\n\ra\n");
        invalid_new_line!(b"\rGET / HTTP/1.1\r\n\r\n");
        out_of_capacity!(b"GET / HTTP/1.1\r\na:b\r\nc:d\r\n\r\n");
    }

    #[test]
    fn incomplete_request() {
        incomplete!(b"");
        incomplete!(b"GET");
        incomplete!(b"GET ");
        incomplete!(b"GET /");
        incomplete!(b"GET / ");
        incomplete!(b"GET / HTT");
        incomplete!(b"GET / HTTP/1.");
        incomplete!(b"GET / HTTP/1.1");
        incomplete!(b"GET / HTTP/1.1\r");
        incomplete!(b"GET / HTTP/1.1\r\n");
        incomplete!(b"GET / HTTP/1.1\r\na");
        incomplete!(b"GET / HTTP/1.1\r\na:");
        incomplete!(b"GET / HTTP/1.1\r\na:b");
        incomplete!(b"GET / HTTP/1.1\r\na:b\r");
        incomplete!(b"GET / HTTP/1.1\r\na:b\r\n");
        incomplete!(b"GET / HTTP/1.1\r\na:b\r\n\r");
    }
}

#[cfg(test)]
mod response {
    use *;

    macro_rules! good {
        ($buf:expr) => {
            good!($buf, |_res| {})
        };
        ($buf:expr, | $res:ident | $body:expr) => {{
            let mut headers = Vec::<HeaderField>::with_capacity(10);
            match Response::parse($buf, &mut headers) {
                Ok(Complete((res, c))) => {
                    assert_eq!(c, $buf.len());
                    closure(res);
                }
                _ => assert!(false),
            }

            fn closure($res: Response) {
                $body
            }
        }};
    }

    macro_rules! fail {
        ($buf:expr, $err:ident) => {{
            let mut headers = Vec::<HeaderField>::with_capacity(1);
            let r = Response::parse($buf, &mut headers);
            assert!(r.is_err());
            assert_eq!(r.err().unwrap(), $err);
        }};
    }

    macro_rules! invalid_version {
        ($parse:expr) => {
            fail!($parse, InvalidVersion)
        };
    }

    macro_rules! invalid_status_code {
        ($parse:expr) => {
            fail!($parse, InvalidStatusCode)
        };
    }

    macro_rules! invalid_reason_phrase {
        ($parse:expr) => {
            fail!($parse, InvalidReasonPhrase)
        };
    }

    macro_rules! out_of_capacity {
        ($parse:expr) => {
            fail!($parse, OutOfCapacity)
        };
    }

    macro_rules! incomplete {
        ($buf:expr) => {{
            let mut headers = Vec::<HeaderField>::with_capacity(10);
            let r = Response::parse($buf, &mut headers);
            assert!(r.is_ok());
            assert!(r.unwrap().is_incomplete());
        }};
    }

    #[test]
    fn simple_response() {
        good!(b"HTTP/1.1 200 OK\r\n\r\n", |res| {
            assert_eq!(res.minor_version, 1);
            assert_eq!(res.status, 200);
            assert_eq!(res.reason, "OK");
            assert_eq!(res.headers.len(), 0);
        })
    }

    #[test]
    fn simple_response_with_headers() {
        good!(b"HTTP/1.1 200 OK\r\na:b\r\nc:d\r\n\r\n", |res| {
            assert_eq!(res.minor_version, 1);
            assert_eq!(res.status, 200);
            assert_eq!(res.reason, "OK");
            assert_eq!(res.headers.len(), 2);
            assert_eq!(res.headers[0].name, "a");
            assert_eq!(res.headers[0].value, "b");
            assert_eq!(res.headers[1].name, "c");
            assert_eq!(res.headers[1].value, "d");
        })
    }

    #[test]
    fn accept_various_new_lines() {
        good!(b"HTTP/1.1 200 OK\n\n");
        good!(b"HTTP/1.1 200 OK\r\n\n");
        good!(b"HTTP/1.1 200 OK\n\r\n");
        good!(b"HTTP/1.1 200 OK\r\na:b\r\n\n");
        good!(b"HTTP/1.1 200 OK\r\na:b\n\n");
        good!(b"HTTP/1.1 200 OK\r\na:b\n\r\n");
    }

    #[test]
    fn skip_front_new_lines() {
        good!(b"\r\nHTTP/1.1 200 OK\r\na:b\n\r\n");
        good!(b"\r\n\r\nHTTP/1.1 200 OK\r\na:b\n\r\n");
        good!(b"\nHTTP/1.1 200 OK\r\na:b\n\r\n");
        good!(b"\n\r\nHTTP/1.1 200 OK\r\na:b\n\r\n");
    }

    #[test]
    fn bad_response() {
        invalid_version!(b"ABC\r\n\r\n");
        invalid_version!(b"HOGE/1.1 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/11.1 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/A.1 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/1.A 200 OK\r\n\r\n");
        invalid_version!(b"HTTP/1.11 200 OK\r\n\r\n");
        invalid_status_code!(b"HTTP/1.1 20 OK\r\na:b\r\n\r\n");
        invalid_status_code!(b"HTTP/1.1 2000 OK\r\na:b\r\n\r\n");
        invalid_status_code!(b"HTTP/1.1 2A00 OK\r\na:b\r\n\r\n");
        invalid_reason_phrase!(b"HTTP/1.1 200 O\x01K\r\na:b\r\n\r\n");
        invalid_reason_phrase!(b"HTTP/1.1 200 O\x01K\r\na:b\r\n\r\n");
        invalid_reason_phrase!(b"HTTP/1.1 200 O\x01K\r\na\x01:b\r\n\r\n");
        out_of_capacity!(b"HTTP/1.1 200 OK\r\na:b\r\nc:d\r\n\r\n");
    }

    #[test]
    fn incomplete_response() {
        incomplete!(b"");
        incomplete!(b"HTT");
        incomplete!(b"HTTP/");
        incomplete!(b"HTTP/1");
        incomplete!(b"HTTP/1.1");
        incomplete!(b"HTTP/1.1 ");
        incomplete!(b"HTTP/1.1 2");
        incomplete!(b"HTTP/1.1 200");
        incomplete!(b"HTTP/1.1 200 ");
        incomplete!(b"HTTP/1.1 200 O");
        incomplete!(b"HTTP/1.1 200 OK");
        incomplete!(b"HTTP/1.1 200 OK\r");
        incomplete!(b"HTTP/1.1 200 OK\r\n");
        incomplete!(b"HTTP/1.1 200 OK\r\na");
        incomplete!(b"HTTP/1.1 200 OK\r\na:");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b\r");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b\r\n");
        incomplete!(b"HTTP/1.1 200 OK\r\na:b\r\n\r");
    }
}
