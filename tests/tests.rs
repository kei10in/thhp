extern crate httpparser;

use httpparser::*;

#[cfg(test)]
mod request {
    use ::*;

    macro_rules! good {
        ($buf: expr) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Request::parse($buf, &mut headers);
                assert!(r.is_ok());
            }
        }
    }

    macro_rules! fail {
        ($buf: expr, $err: ident) => {
            {
                let mut headers = Vec::<HeaderField>::with_capacity(10);
                let r = Request::parse($buf, &mut headers);
                assert!(r.is_err());
                assert_eq!(*r.err().unwrap().kind(), $err);
            }
        }
    }

    macro_rules! invalid_format {
        ($parse: expr) => {
            fail!($parse, InvalidFormat)
        }
    }

    macro_rules! incomplete {
        ($parse: expr) => {
            fail!($parse, Incomplete)
        }
    }

    #[test]
    fn good_request() {
        good!(b"GET / HTTP/1.1\r\n\r\n");
        good!(b"GET / HTTP/1.1\n\n");
        good!(b"GET / HTTP/1.1\r\n\n");
        good!(b"GET / HTTP/1.1\n\r\n");
        good!(b"GET / HTTP/1.1\r\na:b\r\n\r\n");
        good!(b"GET / HTTP/1.1\r\na:b\r\n\n");
        good!(b"GET / HTTP/1.1\r\na:b\n\n");
        good!(b"GET / HTTP/1.1\r\na:b\n\r\n");
    }

    #[test]
    fn bad_request() {
        invalid_format!(b"G\x01ET / HTTP/1.1\r\n\r\n");
        invalid_format!(b"GET /a\x01ef HTTP/1.1\r\n\r\n");
        invalid_format!(b"GET / HOGE\r\n\r\n");
        invalid_format!(b"GET / HTTP/11.1\r\n\r\n");
        invalid_format!(b"GET / HTTP/A.1\r\n\r\n");
        invalid_format!(b"GET / HTTP/1.A\r\n\r\n");
        invalid_format!(b"GET / HTTP/1.A\r\n\r\n");
        invalid_format!(b"GET / HTTP/1.1\r\na\x01b:xyz\r\n\r\n");
        invalid_format!(b"GET / HTTP/1.1\r\nabc:x\x01z\r\n\r\n");
    }

    #[test]
    fn incomplete_request() {
        incomplete!(b"GET");
        incomplete!(b"GET ");
        incomplete!(b"GET /");
        incomplete!(b"GET / ");
        incomplete!(b"GET / HTT");
        incomplete!(b"GET / HTTP/1.");
        incomplete!(b"GET / HTTP/1.1\r\n");
        incomplete!(b"GET / HTTP/1.1\r\na");
        incomplete!(b"GET / HTTP/1.1\r\na:");
        incomplete!(b"GET / HTTP/1.1\r\na:b");
        incomplete!(b"GET / HTTP/1.1\r\na:b\r\n");
    }
}
