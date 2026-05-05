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