pub struct Scanner<'a> {
    buffer: &'a [u8],
    index: usize,
}

impl<'a> Scanner<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> Scanner {
        Scanner {
            buffer: buf,
            index: 0,
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn position(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn rest(&self) -> usize {
        self.buffer.len() - self.index
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.rest() == 0
    }

    #[inline]
    pub fn is_head_of(&self, trunk: &[u8]) -> bool {
        debug_assert!(self.index <= self.buffer.len());
        return unsafe { trunk.starts_with(self.buffer.get_unchecked(self.index..)) };
    }

    #[inline]
    pub fn skip_if(&mut self, needle: &[u8]) -> Option<usize> {
        if unsafe { self.buffer.get_unchecked(self.index..) }.starts_with(needle) {
            self.index += needle.len();
            Some(needle.len())
        } else {
            None
        }
    }

    #[inline]
    pub fn read(&mut self, count: usize) -> Option<&'a [u8]> {
        let r = self.buffer.get(self.index..(self.index + count));
        if r.is_some() {
            self.index += count;
        }

        return r;
    }

    #[inline]
    pub fn read_while<A>(&mut self, mut acceptable: A) -> Option<&'a [u8]>
    where
        A: FnMut(u8) -> bool,
    {
        let s = self.index;
        loop {
            match self.buffer.get(self.index) {
                Some(c) => {
                    if !acceptable(*c) {
                        return Some(unsafe { self.buffer.get_unchecked(s..(self.index)) });
                    }
                    self.index += 1;
                }
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use scanner::*;

    #[test]
    fn test_skip_if() {
        let mut s = Scanner::new(b"HTTP/1.1");

        let r1 = s.skip_if(b"HTTP/");
        assert_eq!(r1, Some(5));
        assert_eq!(s.position(), 5);
    }

    #[test]
    fn test_read() {
        let mut s = Scanner::new(b"HTTP/1.1");
        let r = s.read(5);
        assert_eq!(r, Some(b"HTTP/".as_ref()));
        assert_eq!(s.position(), 5);
    }

    #[test]
    fn test_read_fail() {
        let mut s = Scanner::new(b"ABC");
        let r = s.read(5);
        assert_eq!(r, None);
        assert_eq!(s.position(), 0);
    }

    #[test]
    fn test_read_while() {
        let mut s = Scanner::new(b"GET / ");

        let r1 = s.read_while(|x| b'A' <= x && x <= b'Z');
        assert_eq!(r1, Some(b"GET".as_ref()));
        assert_eq!(s.position(), 3);
    }
}
