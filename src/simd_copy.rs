
/*
High-bandwidth SIMD memory copy utility for optimizing bulk data movement.

This structure leverages CPU vector registers to perform wide memory copies, transferring 
multiple bytes per instruction cycle (16, 32, or 64 bytes at a time depending on supported 
architecture extensions like AVX-512, AVX2, or NEON). It is primarily used to bypass 
the overhead of standard library byte-copy routines in performance-critical scenarios—such 
as rapid serialization of network packets, high-speed buffer shifting in circular queues, 
or frequent memory snapshotting—by maximizing the utilized memory bandwidth and instruction-level 
parallelism.
*/

use std::ptr;

pub struct SimdCopy;

impl SimdCopy {
    #[inline(always)]
    pub unsafe fn copy(src: *const u8, dst: *mut u8, len: usize) {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") && len >= 64 {
                return unsafe { Self::copy_avx512(src, dst, len) };
            }
            if is_x86_feature_detected!("avx2") && len >= 32 {
                return unsafe { Self::copy_avx2(src, dst, len) };
            }
            if is_x86_feature_detected!("sse2") && len >= 16 {
                return unsafe { Self::copy_sse2(src, dst, len) };
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if len >= 16 {
                return unsafe { Self::copy_neon(src, dst, len) };
            }
        }

        unsafe { ptr::copy_nonoverlapping(src, dst, len) };
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f")]
    unsafe fn copy_avx512(src: *const u8, dst: *mut u8, len: usize) {
        use std::arch::x86_64::{_mm512_loadu_si512, _mm512_storeu_si512};
        let mut offset = 0;
        while offset + 64 <= len {
            unsafe {
                let chunk = _mm512_loadu_si512(src.add(offset) as *const _);
                _mm512_storeu_si512(dst.add(offset) as *mut _, chunk);
            }
            offset += 64;
        }
        if offset < len {
            unsafe { ptr::copy_nonoverlapping(src.add(offset), dst.add(offset), len - offset) };
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn copy_avx2(src: *const u8, dst: *mut u8, len: usize) {
        use std::arch::x86_64::{_mm256_loadu_si256, _mm256_storeu_si256};
        let mut offset = 0;
        while offset + 32 <= len {
            unsafe {
                let chunk = _mm256_loadu_si256(src.add(offset) as *const _);
                _mm256_storeu_si256(dst.add(offset) as *mut _, chunk);
            }
            offset += 32;
        }
        if offset < len {
            unsafe { ptr::copy_nonoverlapping(src.add(offset), dst.add(offset), len - offset) };
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn copy_sse2(src: *const u8, dst: *mut u8, len: usize) {
        use std::arch::x86_64::{_mm_loadu_si128, _mm_storeu_si128};
        let mut offset = 0;
        while offset + 16 <= len {
            unsafe {
                let chunk = _mm_loadu_si128(src.add(offset) as *const _);
                _mm_storeu_si128(dst.add(offset) as *mut _, chunk);
            }
            offset += 16;
        }
        if offset < len {
            unsafe { ptr::copy_nonoverlapping(src.add(offset), dst.add(offset), len - offset) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn copy_neon(src: *const u8, dst: *mut u8, len: usize) {
        use std::arch::aarch64::{vld1q_u8, vst1q_u8};
        let mut offset = 0;
        while offset + 16 <= len {
            unsafe {
                let chunk = vld1q_u8(src.add(offset));
                vst1q_u8(dst.add(offset), chunk);
            }
            offset += 16;
        }
        if offset < len {
            unsafe { ptr::copy_nonoverlapping(src.add(offset), dst.add(offset), len - offset) };
        }
    }
}

ftest::test!(simd_copy_tests, {
    test_simd_copy_small_buffer {
        let src = vec![1u8, 2, 3, 4, 5];
        let mut dst = vec![0u8; 5];

        unsafe {
            SimdCopy::copy(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }

        assert_eq!(src, dst);
    }

    test_simd_copy_large_buffer {
        let src = (0..256).map(|i| (i % 255) as u8).collect::<Vec<u8>>();
        let mut dst = vec![0u8; 256];

        unsafe {
            SimdCopy::copy(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }

        assert_eq!(src, dst);
    }

    test_simd_copy_with_offsets {
        let src = vec![10u8; 100];
        let mut dst = vec![0u8; 120];

        unsafe {
            SimdCopy::copy(src.as_ptr().add(10), dst.as_mut_ptr().add(20), 50);
        }

        assert_eq!(&dst[20..70], &src[10..60]);
        assert_eq!(dst[0], 0);
        assert_eq!(dst[119], 0);
    }
});

ftest::bench!(simd_copy_comparison, {
    const SIZE: usize = 8024;

    std_lib_copy {
        let src = vec![0u8; SIZE];
        let mut dst = vec![0u8; SIZE];
        test::black_box(&src);
        test::black_box(&mut dst);
    } -> {
        dst.copy_from_slice(&src);
    }

    simd_manual_copy {
        let src = vec![0u8; SIZE];
        let mut dst = vec![0u8; SIZE];
        test::black_box(&src);
        test::black_box(&mut dst);
    } -> {
        unsafe {
            SimdCopy::copy(src.as_ptr(), dst.as_mut_ptr(), SIZE);
        }
    }
});