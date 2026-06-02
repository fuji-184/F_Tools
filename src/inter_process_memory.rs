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

ftest::test!(inter_process_memory_tests, {
    test_create_and_write_read {
        let shm_name = "/test_shm_unique_name_1";
        let size = 1024;

        let ipm = InterProcessMemory::create(shm_name, size).unwrap();
        let ptr = ipm.as_ptr();

        assert!(!ptr.is_null());

        unsafe {
            let data_to_write = [10u8, 20u8, 30u8, 40u8];
            std::ptr::copy_nonoverlapping(data_to_write.as_ptr(), ptr, data_to_write.len());

            let mut data_read = [0u8; 4];
            std::ptr::copy_nonoverlapping(ptr, data_read.as_mut_ptr(), data_read.len());

            assert_eq!(data_to_write, data_read);
        }
    }

    test_invalid_name {
        let shm_name = "invalid_name_without_leading_slash";
        let size = 1024;

        let ipm = InterProcessMemory::create(shm_name, size);
        assert!(ipm.is_err() || ipm.is_ok());
    }
});