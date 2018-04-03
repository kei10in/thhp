pub struct Scanner<'a> {
    buffer: &'a [u8],
}

impl<'a> Scanner<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> Scanner {
        Scanner { buffer: buf }
    }

    #[inline]
    pub fn rest(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.rest() == 0
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
    pub fn skip_if(&mut self, needle: &[u8]) -> Option<()> {
        if self.buffer.starts_with(needle) {
            self.buffer = unsafe { self.buffer.get_unchecked(needle.len()..) };
            Some(())
        } else {
            None
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
    pub fn read_while<A>(&mut self, mut acceptable: A) -> Option<&'a [u8]>
    where
        A: FnMut(u8) -> bool,
    {
        let mut i = 0;
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
        let result = unsafe { self.buffer.get_unchecked(..i) };
        self.buffer = unsafe { self.buffer.get_unchecked(i..) };

        return Some(result);
    }
}

#[cfg(test)]
mod tests {
    use scanner::*;

    #[test]
    fn test_skip_if() {
        let mut s = Scanner::new(b"HTTP/1.1");

        let r1 = s.skip_if(b"HTTP/");
        assert_eq!(r1, Some(()));
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
        let mut s = Scanner::new(b"GET / ");

        let r1 = s.read_while(|x| b'A' <= x && x <= b'Z');
        assert_eq!(r1, Some(b"GET".as_ref()));
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
}
