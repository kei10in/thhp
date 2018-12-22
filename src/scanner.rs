#[cfg(thhp_enable_sse42)]
use crate::simd;

#[cfg(thhp_enable_sse42)]
pub use crate::simd::CharRanges;

pub struct Scanner<'a> {
    buffer: &'a [u8],
}

impl<'a> Scanner<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> Scanner {
        Scanner { buffer: buf }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn peek(&self, index: usize) -> Option<&u8> {
        self.buffer.get(index)
    }

    #[inline]
    pub unsafe fn skip_unchecked(&mut self, count: usize) {
        self.buffer = self.buffer.get_unchecked(count..);
    }

    #[inline]
    pub fn is_head_of(&self, trunk: &[u8]) -> bool {
        return trunk.starts_with(self.buffer);
    }

    #[inline]
    pub fn skip_if(&mut self, needle: &[u8]) -> bool {
        if self.buffer.starts_with(needle) {
            self.buffer = unsafe { self.buffer.get_unchecked(needle.len()..) };
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn read(&mut self, count: usize) -> Option<&'a [u8]> {
        let r = self.buffer.get(..count);
        if r.is_some() {
            self.buffer = unsafe { self.buffer.get_unchecked(count..) };
        }

        return r;
    }

    #[inline]
    unsafe fn read_unchecked(&mut self, count: usize) -> &'a [u8] {
        let result = self.buffer.get_unchecked(..count);
        self.buffer = self.buffer.get_unchecked(count..);
        return result;
    }

    #[inline]
    pub fn read_while<A>(&mut self, acceptable: A) -> Option<&'a [u8]>
    where
        A: FnMut(u8) -> bool,
    {
        self.read_while_continue_with(0, acceptable)
    }

    #[cfg(thhp_enable_sse42)]
    #[inline]
    pub fn read_while_fast<A>(&mut self, range: &CharRanges, acceptable: A) -> Option<&'a [u8]>
    where
        A: FnMut(u8) -> bool,
    {
        let (v, found) = simd::index_of_range_or_last_16bytes_boundary(self.buffer, range);
        if found {
            unsafe { Some(self.read_unchecked(v)) }
        } else {
            self.read_while_continue_with(v, acceptable)
        }
    }

    #[inline]
    pub fn read_while_continue_with<A>(
        &mut self,
        cont: usize,
        mut acceptable: A,
    ) -> Option<&'a [u8]>
    where
        A: FnMut(u8) -> bool,
    {
        let mut i = cont;
        loop {
            if let Some(val) = self.buffer.get(i..i + 8) {
                unsafe {
                    if !acceptable(*val.get_unchecked(0)) {
                        break;
                    } else if !acceptable(*val.get_unchecked(1)) {
                        i += 1;
                        break;
                    } else if !acceptable(*val.get_unchecked(2)) {
                        i += 2;
                        break;
                    } else if !acceptable(*val.get_unchecked(3)) {
                        i += 3;
                        break;
                    } else if !acceptable(*val.get_unchecked(4)) {
                        i += 4;
                        break;
                    } else if !acceptable(*val.get_unchecked(5)) {
                        i += 5;
                        break;
                    } else if !acceptable(*val.get_unchecked(6)) {
                        i += 6;
                        break;
                    } else if !acceptable(*val.get_unchecked(7)) {
                        i += 7;
                        break;
                    } else {
                        i += 8;
                    }
                }
            } else {
                loop {
                    match self.buffer.get(i) {
                        Some(c) => {
                            if acceptable(*c) {
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        None => return None,
                    }
                }
                break;
            }
        }

        debug_assert!(i <= self.buffer.len());
        return unsafe { Some(self.read_unchecked(i)) };
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::*;

    #[test]
    fn test_skip_if() {
        let mut s = Scanner::new(b"HTTP/1.1");

        let r1 = s.skip_if(b"HTTP/");
        assert_eq!(r1, true);
    }

    #[test]
    fn test_read() {
        let mut s = Scanner::new(b"HTTP/1.1");
        let r = s.read(5);
        assert_eq!(r, Some(b"HTTP/".as_ref()));
    }

    #[test]
    fn test_read_fail() {
        let mut s = Scanner::new(b"ABC");
        let r = s.read(5);
        assert_eq!(r, None);
    }

    #[test]
    fn test_read_while() {
        macro_rules! check_read_while {
            ($buf:expr, $expect:expr) => {
                let mut s = Scanner::new($buf);
                let r = s.read_while(|x| b'A' <= x && x <= b'Z');
                assert!(r.is_some());
                assert_eq!(r.unwrap(), $expect.as_ref());
            };
        }

        check_read_while!(b"A ", b"A");
        check_read_while!(b"AB ", b"AB");
        check_read_while!(b"ABC ", b"ABC");
        check_read_while!(b"ABCD ", b"ABCD");
        check_read_while!(b"ABCDE ", b"ABCDE");
        check_read_while!(b"ABCDEF ", b"ABCDEF");
        check_read_while!(b"ABCDEFG ", b"ABCDEFG");
        check_read_while!(b"ABCDEFGH ", b"ABCDEFGH");
        check_read_while!(b"ABCDEFGHI ", b"ABCDEFGHI");
    }

    #[test]
    fn test_read_3_chars_by_read_while() {
        let mut s = Scanner::new(b"GET / ");

        let r1 = s.read_while(|x| b'A' <= x && x <= b'Z');
        assert_eq!(r1, Some(b"GET".as_ref()));
    }

    #[test]
    fn test_read_10_chars_by_read_while() {
        let mut s = Scanner::new(b"HELLOWORLD!");

        let r1 = s.read_while(|x| b'A' <= x && x <= b'Z');
        assert_eq!(r1, Some(b"HELLOWORLD".as_ref()));
    }

    mod simd {
        #![cfg(thhp_enable_sse42)]

        use crate::scanner::*;

        macro_rules! check {
            ($buf:expr, $expect:expr) => {
                let mut s = Scanner::new($buf);
                let r = s.read_while_fast(&b";;".into(), |x| x != b';');
                assert_eq!(r, $expect);
            };
        }

        #[test]
        fn test_read_while_fast() {
            check!(b"a", None);
            check!(b"a;", Some(b"a".as_ref()));
            check!(b"aaaaaaaaaaaaaaa;", Some(b"aaaaaaaaaaaaaaa".as_ref()));
            check!(b"aaaaaaaaaaaaaaaa;", Some(b"aaaaaaaaaaaaaaaa".as_ref()));
            check!(
                b"aaaaaaaaaaaaaaaa;aaaaaaaaaaaaaaa",
                Some(b"aaaaaaaaaaaaaaaa".as_ref())
            );
            check!(
                b"aaaaaaaaaaaaaaaaaaaaaaaaaa;aaaaa",
                Some(b"aaaaaaaaaaaaaaaaaaaaaaaaaa".as_ref())
            );
        }
    }
}
