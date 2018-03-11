extern crate httpparser;

use httpparser::*;

macro_rules! fail{
        ($parse: expr, $err: ident) => {
            {
                let r = $parse;
                assert!(r.is_err());
                assert_eq!(*r.err().unwrap().kind(), $err);
            }
        }
    }

macro_rules! invalid_format{
        ($parse: expr) => {
            fail!($parse, InvalidFormat)
        }
    }

macro_rules! incomplete{
        ($parse: expr) => {
            fail!($parse, Incomplete)
        }
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

#[test]
fn bad_request() {
    let mut headers = Vec::<HeaderField>::with_capacity(10);
    invalid_format!(Request::parse(b"G\x01ET / HTTP/1.1\r\n\r\n", &mut headers));
    invalid_format!(Request::parse(
        b"GET /a\x01ef HTTP/1.1\r\n\r\n",
        &mut headers
    ));
    invalid_format!(Request::parse(b"GET / HOGE\r\n\r\n", &mut headers));
    invalid_format!(Request::parse(b"GET / HTTP/11.1\r\n\r\n", &mut headers));
    invalid_format!(Request::parse(b"GET / HTTP/A.1\r\n\r\n", &mut headers));
    invalid_format!(Request::parse(b"GET / HTTP/1.A\r\n\r\n", &mut headers));
    invalid_format!(Request::parse(b"GET / HTTP/1.A\r\n\r\n", &mut headers));
    invalid_format!(Request::parse(
        b"GET / HTTP/1.1\r\na\x01b:xyz\r\n\r\n",
        &mut headers
    ));
    invalid_format!(Request::parse(
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
