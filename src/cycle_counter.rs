pub struct CycleResult {
    pub cycles: u64,
    pub core_id: u32,
}

pub struct CycleCounter {
    pub start: u64,
    pub start_core: u32,
}

impl CycleCounter {
    #[inline(always)]
    pub fn start() -> Self {
        let (tsc, core) = Self::read_raw();
        Self {
            start: tsc,
            start_core: core,
        }
    }

    #[inline(always)]
    pub fn elapsed(&self) -> CycleResult {
        let (tsc_end, core) = Self::read_raw();
        CycleResult {
            cycles: tsc_end.wrapping_sub(self.start),
            core_id: core,
        }
    }

    #[inline(always)]
    pub fn is_same_core(&self, result: &CycleResult) -> bool {
        self.start_core == result.core_id
    }

    #[inline(always)]
    fn read_raw() -> (u64, u32) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_rdtscp, _mm_lfence};
            let mut aux = 0u32;
            _mm_lfence();
            let tsc = _rdtscp(&mut aux as *mut u32);
            (tsc, aux)
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            let vct: u64;
            std::arch::asm!("isb", "mrs {}, cntvct_el0", out(reg) vct, options(nostack));
            (vct, 0) 
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64, 0)
        }
    }
}