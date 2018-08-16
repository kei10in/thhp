#![cfg(feature = "nightly")]
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::convert::*;

use packed_simd::*;

pub struct CharRanges {
    value: u8x16,
    len: i32,
}

impl CharRanges {
    pub const fn new2(v: &[u8; 2]) -> CharRanges {
        CharRanges {
            value: u8x16::new(v[0], v[1], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            len: 2,
        }
    }

    pub const fn new6(v: &[u8; 6]) -> CharRanges {
        CharRanges {
            value: u8x16::new(
                v[0], v[1], v[2], v[3], v[4], v[5], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ),
            len: 6,
        }
    }
}

impl<'a> Into<CharRanges> for &'a [u8; 2] {
    fn into(self) -> CharRanges {
        CharRanges::new2(self)
    }
}

impl<'a> Into<CharRanges> for &'a [u8; 6] {
    fn into(self) -> CharRanges {
        CharRanges::new6(self)
    }
}

#[inline]
pub fn index_of_range_or_last_16bytes_boundary(buffer: &[u8], range: &CharRanges) -> (usize, bool) {
    let mut i = 0;
    loop {
        if buffer.len() - i < 16 {
            return (i, false);
        }
        let v = unsafe { buffer.get_unchecked(i..i + 16) };
        let idx = find_fast(v, range);
        if idx != 16 {
            i += idx;
            return (i, true);
        }
        i += 16;
    }
}

#[inline]
fn find_fast(buffer: &[u8], range: &CharRanges) -> usize {
    debug_assert!(buffer.len() <= 16);

    unsafe {
        let a = __m128i::from_bits(range.value);
        let b = __m128i::from_bits(u8x16::from_slice_unaligned_unchecked(buffer));

        let i = _mm_cmpestri(
            a,
            range.len,
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
        ($buf:expr, $range:expr, $index:expr, $found:expr) => {
            let v = index_of_range_or_last_16bytes_boundary($buf, &$range.into());
            assert_eq!(v, ($index, $found));
        };
    }

    #[test]
    fn test_index_of_range_or_last_16bytes_boundary_found() {
        check!(b"1aaaaaaabbbbbbbb", b"09", 0, true);
        check!(b"aaaaaaa1bbbbbbbb", b"09", 7, true);
        check!(b"aaaaaaaabbbbbbb1", b"09", 15, true);
        check!(b"aaaaaaaabbbbbbbb1cccccccdddddddd", b"09", 16, true);
        check!(b"aaaaaaaabbbbbbbbccccccc1dddddddd", b"09", 23, true);
        check!(b"aaaaaaaabbbbbbbbccccccccddddddd1", b"09", 31, true);
    }

    #[test]
    fn test_index_of_range_or_last_16bytes_boundary_not_found() {
        check!(b"aaaaaaaabbbbbbbb", b"09", 16, false);
        check!(b"aaaaaaaabbbbbbbbccccccccdddddddd", b"09", 32, false);
    }

    #[test]
    fn test_index_of_range_or_last_16bytes_boundary_returns_last_16bytes_boundary() {
        check!(b"1aaaaaaabbbbbbb", b"09", 0, false);
        check!(b"aaaaaaa1bbbbbbb", b"09", 0, false);
        check!(b"aaaaaaaabbbbbb1", b"09", 0, false);
        check!(b"aaaaaaaabbbbbbbb1cccccccddddddd", b"09", 16, false);
        check!(b"aaaaaaaabbbbbbbbccccccc1ddddddd", b"09", 16, false);
        check!(b"aaaaaaaabbbbbbbbccccccccdddddd1", b"09", 16, false);
    }
}
