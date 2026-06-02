
/*
Low-level operating system primitive for building ultra-fast custom synchronization abstractions.

This mechanism acts as the foundation for construct elements like custom mutexes, condition 
variables, and thread primitives. It prevents threads from consuming CPU cycles by putting 
them into an efficient sleep state directly via the OS kernel when an expected condition 
is not met, and allows other threads to instantly wake them up once the resource becomes free.
*/

use std::sync::atomic::AtomicU32;
use std::ptr;

pub struct Futex;

impl Futex {
    pub fn wait(addr: &AtomicU32, expected: u32) {
        unsafe {
            libc::syscall(
                libc::SYS_futex,
                addr as *const AtomicU32,
                libc::FUTEX_WAIT | libc::FUTEX_PRIVATE_FLAG,
                expected,
                ptr::null::<libc::timespec>(), 
                ptr::null::<u32>(),
                0,
            );
        }
    }

    pub fn wake(addr: &AtomicU32, count: i32) {
        unsafe {
            libc::syscall(
                libc::SYS_futex,
                addr as *const AtomicU32,
                libc::FUTEX_WAKE | libc::FUTEX_PRIVATE_FLAG,
                count,
                ptr::null::<libc::timespec>(),
                ptr::null::<u32>(),
                0,
            );
        }
    }
}

ftest::test!(futex_tests, {
    test_futex_wait_and_wake {
        let addr = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let addr_clone = addr.clone();

        let handle = std::thread::spawn(move || {
            Futex::wait(&addr_clone, 0);
            assert_eq!(addr_clone.load(std::sync::atomic::Ordering::Acquire), 1);
        });

        std::thread::sleep(std::time::Duration::from_millis(10));
        addr.store(1, std::sync::atomic::Ordering::Release);
        Futex::wake(&addr, 1);

        assert!(handle.join().is_ok());
    }

    test_futex_wait_no_block_if_value_mismatch {
        let addr = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(42));
        let start = std::time::Instant::now();

        Futex::wait(&addr, 0);

        assert!(start.elapsed() < std::time::Duration::from_millis(10));
    }
});