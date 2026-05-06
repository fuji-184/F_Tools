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