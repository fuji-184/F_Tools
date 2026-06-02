
/*
Thread synchronization barrier to coordinate phase-based execution across parallel workers.

This mechanism forces a designated number of threads to pause and align at a specific 
checkpoint before any of them are permitted to advance further. It is primarily used 
to synchronize cyclical, multi-step operations—such as parallel game loop rendering stages, 
multi-threaded matrix math computations, or batch data processing pipelines—ensuring that 
no worker begins the next stage until all workers have finished the current one.
*/

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

ftest::test!(futex_thread_pause_tests, {
    test_barrier_synchronization {
        let barrier = std::sync::Arc::new(FutexThreadPause::new(3));
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut handles = Vec::new();

        for _ in 0..3 {
            let b = barrier.clone();
            let c = counter.clone();
            handles.push(std::thread::spawn(move || {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                b.wait();
                assert_eq!(c.load(std::sync::atomic::Ordering::SeqCst), 3);
            }));
        }

        for handle in handles {
            assert!(handle.join().is_ok());
        }
    }

    test_multiple_generations {
        let barrier = std::sync::Arc::new(FutexThreadPause::new(2));
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut handles = Vec::new();

        for _ in 0..2 {
            let b = barrier.clone();
            let c = counter.clone();
            handles.push(std::thread::spawn(move || {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                b.wait();

                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                b.wait();
            }));
        }

        for handle in handles {
            assert!(handle.join().is_ok());
        }

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 4);
    }
});