use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::slice;
use std::marker::PhantomData;

pub struct MemMap {
    pub ptr: *mut u8,
    pub len: usize,
}

impl MemMap {
    pub fn open<P: AsRef<std::path::Path>>(path: P, writable: bool, size: Option<usize>) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(writable)
            .create(writable && size.is_some())
            .open(path)?;
            
        if let Some(s) = size {
            if writable {
                file.set_len(s as u64)?;
            }
        }
        
        let len = file.metadata()?.len() as usize;
        if len == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "File is empty"));
        }

        let prot = if writable {
            libc::PROT_READ | libc::PROT_WRITE
        } else {
            libc::PROT_READ
        };

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                prot,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self { ptr: ptr as *mut u8, len })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
    
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        let bytes = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        std::str::from_utf8(bytes)
    }

    pub fn as_str_unchecked(&self) -> &str {
        unsafe {
            let bytes = std::slice::from_raw_parts(self.ptr, self.len);
            std::str::from_utf8_unchecked(bytes)
        }
    }

    pub fn flush(&self) -> std::io::Result<()> {
        let res = unsafe { libc::msync(self.ptr as *mut libc::c_void, self.len, libc::MS_SYNC) };
        if res == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) }
    }
    
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for MemMap {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, self.len);
        }
    }
}

unsafe impl Send for MemMap {}
unsafe impl Sync for MemMap {}



/// harus digunakan bersama dengan #[repr(C)] atau #[repr(transparent)]
pub unsafe trait Pod: Copy + 'static {}

pub struct StructView<'a, T: Pod> {
    data: &'a T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Pod> StructView<'a, T> {
    pub fn new(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() < std::mem::size_of::<T>() {
            return None;
        }

        let ptr = bytes.as_ptr() as *const T;

        if (ptr as usize) % std::mem::align_of::<T>() != 0 {
            return None;
        }

        Some(Self {
            data: unsafe { &*ptr },
            _marker: PhantomData,
        })
    }

    pub fn get(&self) -> &T {
        self.data
    }
}