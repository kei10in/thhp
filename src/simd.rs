#![cfg(feature = "nightly")]
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::simd::*;

#[inline]
pub fn index_of_range_or_last_16bytes_boundary(buffer: &[u8], range: &[u8]) -> usize {
    let mut i = 0;
    loop {
        if buffer.len() - i < 16 {
            return i;
        }
        let v = unsafe { buffer.get_unchecked(i..i + 16) };
        let idx = find_fast(v, range);
        if idx < 16 {
            i += idx;
            return i;
        }
        i += 16;
    }
}

#[inline]
fn find_fast(buffer: &[u8], range: &[u8]) -> usize {
    debug_assert!(buffer.len() <= 16);
    debug_assert!(range.len() <= 16);

    unsafe {
        let a = __m128i::from_bits(u8x16::load_unaligned_unchecked(range));
        let b = __m128i::from_bits(u8x16::load_unaligned_unchecked(buffer));

        let i = _mm_cmpestri(
            a,
            range.len() as i32,
            b,
            buffer.len() as i32,
            _SIDD_UBYTE_OPS | _SIDD_CMP_RANGES | _SIDD_LEAST_SIGNIFICANT,
        );

        return i as usize;
    }
}

#[cfg(test)]
mod tests {
    use simd::*;

    macro_rules! check {
        ($buf:expr, $range:expr, $index:expr) => {
            let v = index_of_range_or_last_16bytes_boundary($buf, $range);
            assert_eq!(v, $index);
        };
    }

    #[test]
    fn test_index_of_range_or_last_16bytes_boundary_found() {
        check!(b"1aaaaaaabbbbbbbb", b"09", 0);
        check!(b"aaaaaaa1bbbbbbbb", b"09", 7);
        check!(b"aaaaaaaabbbbbbb1", b"09", 15);
        check!(b"aaaaaaaabbbbbbbb1cccccccdddddddd", b"09", 16);
        check!(b"aaaaaaaabbbbbbbbccccccc1dddddddd", b"09", 23);
        check!(b"aaaaaaaabbbbbbbbccccccccddddddd1", b"09", 31);
    }

    #[test]
    fn test_index_of_range_or_last_16bytes_boundary_not_found() {
        check!(b"aaaaaaaabbbbbbbb", b"09", 16);
        check!(b"aaaaaaaabbbbbbbbccccccccdddddddd", b"09", 32);
    }

    #[test]
    fn test_index_of_range_or_last_16bytes_boundary_returns_last_16bytes_boundary() {
        check!(b"1aaaaaaabbbbbbb", b"09", 0);
        check!(b"aaaaaaa1bbbbbbb", b"09", 0);
        check!(b"aaaaaaaabbbbbb1", b"09", 0);
        check!(b"aaaaaaaabbbbbbbb1cccccccddddddd", b"09", 16);
        check!(b"aaaaaaaabbbbbbbbccccccc1ddddddd", b"09", 16);
        check!(b"aaaaaaaabbbbbbbbccccccccdddddd1", b"09", 16);
    }
}
