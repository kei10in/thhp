use arrayvec::ArrayVec;

use errors::*;

use HeaderField;
use HeaderFieldCollection;

impl<'buffer> HeaderFieldCollection<'buffer> for ArrayVec<[HeaderField<'buffer>; 16]> {
    fn push(&mut self, header_field: HeaderField<'buffer>) -> Result<()> {
        if self.len() == self.capacity() {
            Err(OutOfCapacity.into())
        } else {
            self.push(header_field);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;

    use *;

    #[test]
    fn test_arrayvec_for_headers() {
        let buf = b"GET / HTTP/1.1\r\na:b\r\n\r\n";
        let mut headers = ArrayVec::<[HeaderField; 16]>::new();
        match Request::parse(buf, &mut headers) {
            Ok(Complete((res, _))) => assert_eq!(res.headers.len(), 1),
            _ => assert!(false),
        }
    }
}
