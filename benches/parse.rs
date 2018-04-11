#![feature(test)]

extern crate httparse;
extern crate picohttpparser_sys as pico;
extern crate thhp;

extern crate test;

const REQ: &'static [u8] = b"\
GET /wp-content/uploads/2010/03/hello-kitty-darth-vader-pink.jpg HTTP/1.1\r\n\
Host: www.kittyhell.com\r\n\
User-Agent: Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: ja,en-us;q=0.7,en;q=0.3\r\n\
Accept-Encoding: gzip,deflate\r\n\
Accept-Charset: Shift_JIS,utf-8;q=0.7,*;q=0.7\r\n\
Keep-Alive: 115\r\n\
Connection: keep-alive\r\n\
Cookie: wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral\r\n\r\n";

#[bench]
fn bench_picohttpparser(b: &mut test::Bencher) {
    use std::ptr;

    let mut method: *const _ = ptr::null_mut();
    let mut method_len = 0;
    let mut path: *const _ = ptr::null_mut();
    let mut path_len = 0;
    let mut minor_version = 0;
    let mut headers = [pico::phr_header::default(); 16];
    let mut headers_len = headers.len();
    let prev_buf_len = 0;

    b.iter(|| {
        let ret = unsafe {
            pico::phr_parse_request(
                REQ.as_ptr() as *const _,
                REQ.len(),
                &mut method,
                &mut method_len,
                &mut path,
                &mut path_len,
                &mut minor_version,
                headers.as_mut_ptr(),
                &mut headers_len,
                prev_buf_len,
            )
        };
        assert_eq!(ret, REQ.len() as i32);
    });
    b.bytes = REQ.len() as u64;
}

#[bench]
fn bench_httparse(b: &mut test::Bencher) {
    let mut headers = [httparse::Header {
        name: "",
        value: &[],
    }; 16];
    let mut req = httparse::Request::new(&mut headers);
    b.iter(|| {
        assert_eq!(
            req.parse(REQ).unwrap(),
            httparse::Status::Complete(REQ.len())
        );
    });
    b.bytes = REQ.len() as u64;
}

#[bench]
fn bench_thhp(b: &mut test::Bencher) {
    let mut headers = Vec::<thhp::HeaderField>::with_capacity(16);
    b.iter(|| {
        headers.clear();
        let res = thhp::Request::parse(REQ, &mut headers);
        assert!(res.is_ok());
    });
    b.bytes = REQ.len() as u64;
}
