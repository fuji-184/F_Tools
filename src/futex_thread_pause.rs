use std::sync::atomic::{AtomicI32, Ordering};
use libc::{syscall, SYS_futex};

const FUTEX_WAIT_PRIVATE: libc::c_int = 128;
const FUTEX_WAKE_PRIVATE: libc::c_int = 129;

pub struct FutexThreadPause {
    count: AtomicI32,
    num_threads: i32,
    generation: AtomicI32,
}

impl FutexThreadPause {
    pub fn new(num_threads: usize) -> Self {
        Self {
            count: AtomicI32::new(num_threads as i32),
            num_threads: num_threads as i32,
            generation: AtomicI32::new(0),
        }
    }

    pub fn wait(&self) {
        let genn = self.generation.load(Ordering::Acquire);
        let current_count = self.count.fetch_sub(1, Ordering::AcqRel);

        if current_count == 1 {
            self.count.store(self.num_threads, Ordering::Release);
            self.generation.fetch_add(1, Ordering::Release);
            
            unsafe {
                syscall(
                    SYS_futex,
                    self.generation.as_ptr(),
                    FUTEX_WAKE_PRIVATE,
                    i32::MAX,
                    std::ptr::null::<libc::timespec>(),
                    std::ptr::null::<i32>(),
                    0,
                );
            }
        } else {
            while self.generation.load(Ordering::Acquire) == genn {
                unsafe {
                    syscall(
                        SYS_futex,
                        self.generation.as_ptr(),
                        FUTEX_WAIT_PRIVATE,
                        genn,
                        std::ptr::null::<libc::timespec>(),
                        std::ptr::null::<i32>(),
                        0,
                    );
                }
            }
        }
    }
}