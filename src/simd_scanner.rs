
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

pub struct SimdScanner;

impl SimdScanner {
    pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
        let len = haystack.len();
        if len < 32 {
            return haystack.iter().position(|&b| b == needle);
        }

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { Self::scan_avx2(haystack, needle) };
            }
            if is_x86_feature_detected!("sse4.2") {
                return unsafe { Self::scan_sse42(haystack, needle) };
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return unsafe { Self::scan_neon(haystack, needle) };
            }
        }

        haystack.iter().position(|&b| b == needle)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn scan_avx2(haystack: &[u8], needle: u8) -> Option<usize> {
        let mut i = 0;
        let len = haystack.len();
        let needle_vec = _mm256_set1_epi8(needle as i8);

        while i + 32 <= len {
            let chunk = _mm256_loadu_si256(haystack.as_ptr().add(i) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(chunk, needle_vec);
            let mask = _mm256_movemask_epi8(cmp);

            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 32;
        }
        haystack[i..].iter().position(|&b| b == needle).map(|pos| i + pos)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse4.2")]
    unsafe fn scan_sse42(haystack: &[u8], needle: u8) -> Option<usize> {
        let mut i = 0;
        let len = haystack.len();
        let needle_vec = _mm_set1_epi8(needle as i8);

        while i + 16 <= len {
            let chunk = _mm_loadu_si128(haystack.as_ptr().add(i) as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, needle_vec);
            let mask = _mm_movemask_epi8(cmp);

            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 16;
        }
        haystack[i..].iter().position(|&b| b == needle).map(|pos| i + pos)
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn scan_neon(haystack: &[u8], needle: u8) -> Option<usize> {
        let mut i = 0;
        let len = haystack.len();
        let needle_vec = vdupq_n_u8(needle);

        while i + 16 <= len {
            let chunk = unsafe { vld1q_u8(haystack.as_ptr().add(i)) };
            let cmp = vceqq_u8(chunk, needle_vec);
            
            let high = vgetq_lane_u64(vreinterpretq_u64_u8(cmp), 1);
            let low = vgetq_lane_u64(vreinterpretq_u64_u8(cmp), 0);

            if low != 0 {
                return Some(i + (low.trailing_zeros() >> 3) as usize);
            }
            if high != 0 {
                return Some(i + 8 + (high.trailing_zeros() >> 3) as usize);
            }
            i += 16;
        }
        haystack[i..].iter().position(|&b| b == needle).map(|pos| i + pos)
    }
}

ftest::test!(simd_scanner_tests, {
    test_scan_not_found {
        let haystack = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = SimdScanner::find_byte(&haystack, 99);
        assert!(result.is_none());
    }

    test_scan_small_buffer {
        let haystack = vec![1u8, 2, 3, 4, 5, 42, 6, 7];
        let result = SimdScanner::find_byte(&haystack, 42);
        assert_eq!(result, Some(5));
    }

    test_scan_large_buffer_exact_simd_boundary {
        let mut haystack = vec![0u8; 32];
        haystack[31] = 42;
        let result = SimdScanner::find_byte(&haystack, 42);
        assert_eq!(result, Some(31));
    }

    test_scan_large_buffer_first_element {
        let mut haystack = vec![0u8; 100];
        haystack[0] = 42;
        let result = SimdScanner::find_byte(&haystack, 42);
        assert_eq!(result, Some(0));
    }

    test_scan_large_buffer_tail_fallback {
        let mut haystack = vec![0u8; 40];
        haystack[38] = 42;
        let result = SimdScanner::find_byte(&haystack, 42);
        assert_eq!(result, Some(38));
    }

    test_scan_multiple_matches {
        let mut haystack = vec![0u8; 64];
        haystack[10] = 42;
        haystack[20] = 42;
        let result = SimdScanner::find_byte(&haystack, 42);
        assert_eq!(result, Some(10));
    }
});