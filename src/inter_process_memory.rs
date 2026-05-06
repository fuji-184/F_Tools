use libc::{shm_open, ftruncate, mmap, munmap, close, shm_unlink};
use libc::{O_CREAT, O_RDWR, PROT_READ, PROT_WRITE, MAP_SHARED, S_IRUSR, S_IWUSR};
use std::io;
use std::ptr;
use std::ffi::CString;

pub struct InterProcessMemory {
    name: CString,
    ptr: *mut libc::c_void,
    size: usize,
}

impl InterProcessMemory {
    pub fn create(name: &str, size: usize) -> io::Result<Self> {
        let shm_name = CString::new(name).unwrap();
        unsafe {
            let fd = shm_open(shm_name.as_ptr(), O_CREAT | O_RDWR, (S_IRUSR | S_IWUSR) as libc::mode_t);
            if fd < 0 { return Err(io::Error::last_os_error()); }

            if ftruncate(fd, size as libc::off_t) < 0 {
                close(fd);
                return Err(io::Error::last_os_error());
            }

            let ptr = mmap(ptr::null_mut(), size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
            close(fd);

            if ptr == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            Ok(Self { name: shm_name, ptr, size })
        }
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr as *mut u8
    }
}

impl Drop for InterProcessMemory {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr, self.size);
            shm_unlink(self.name.as_ptr());
        }
    }
}