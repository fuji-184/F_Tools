
/*
High-performance file I/O framework for zero-copy data parsing and storage persistence.

This mechanism maps storage files directly into the process's virtual memory layout and overlays 
structured types safely across the resulting byte streams. It is primarily used to optimize heavy 
disk operations—such as reading large database index files, updating persistence logs, or parsing 
binary payload records—by treating file contents as a raw memory slice and mapping structured 
C-compatible data types onto them without executing heap allocations or runtime copy overhead.
*/

use std::fs::OpenOptions;
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

ftest::test!(mem_map_and_struct_view_tests, {
    test_mem_map_write_and_read {
        let dir = std::env::temp_dir();
        let path = dir.join("test_mem_map_1.bin");
        
        {
            let mut mmap = MemMap::open(&path, true, Some(128)).unwrap();
            assert_eq!(mmap.len(), 128);
            
            let slice = mmap.as_mut_slice();
            slice[0] = b'H';
            slice[1] = b'i';
            mmap.flush().unwrap();
        }

        {
            let mmap = MemMap::open(&path, false, None).unwrap();
            assert_eq!(mmap.len(), 128);
            
            let slice = mmap.as_slice();
            assert_eq!(slice[0], b'H');
            assert_eq!(slice[1], b'i');
            
            assert_eq!(&mmap.as_str_unchecked()[0..2], "Hi");
            assert_eq!(mmap.as_str().unwrap()[0..2].to_string(), "Hi".to_string());
        }

        let _ = std::fs::remove_file(path);
    }

    test_mem_map_empty_file_error {
        let dir = std::env::temp_dir();
        let path = dir.join("test_mem_map_empty.bin");
        std::fs::File::create(&path).unwrap();

        let mmap = MemMap::open(&path, false, None);
        assert!(mmap.is_err());

        let _ = std::fs::remove_file(path);
    }

    test_struct_view_success {
        #[derive(Copy, Clone, PartialEq, Debug)]
        #[repr(C)]
        struct TestStruct {
            a: u32,
            b: u64,
        }
        unsafe impl Pod for TestStruct {}

        let mut bytes = vec![0u8; 32];
        let ptr = bytes.as_mut_ptr();
        
        let offset = if (ptr as usize) % std::mem::align_of::<TestStruct>() == 0 {
            0
        } else {
            std::mem::align_of::<TestStruct>() - ((ptr as usize) % std::mem::align_of::<TestStruct>())
        };

        let aligned_slice = &mut bytes[offset..(offset + std::mem::size_of::<TestStruct>())];
        
        let sample = TestStruct { a: 1337, b: 424242 };
        unsafe {
            std::ptr::copy_nonoverlapping(
                &sample as *const TestStruct as *const u8,
                aligned_slice.as_mut_ptr(),
                std::mem::size_of::<TestStruct>(),
            );
        }

        let view = StructView::<TestStruct>::new(aligned_slice);
        assert!(view.is_some());
        assert_eq!(view.unwrap().get(), &sample);
    }

    test_struct_view_insufficient_bytes {
        #[derive(Copy, Clone)]
        #[repr(C)]
        struct TestStruct {
            a: u64,
        }
        unsafe impl Pod for TestStruct {}

        let bytes = vec![0u8; 4];
        let view = StructView::<TestStruct>::new(&bytes);
        assert!(view.is_none());
    }

    test_struct_view_misaligned {
        #[derive(Copy, Clone)]
        #[repr(C)]
        struct TestStruct {
            a: u64,
        }
        unsafe impl Pod for TestStruct {}

        let bytes = vec![0u8; 32];
        let ptr = bytes.as_ptr();
        
        let mut unaligned_idx = 0;
        for i in 0..8 {
            if ((ptr as usize) + i) % std::mem::align_of::<TestStruct>() != 0 {
                unaligned_idx = i;
                break;
            }
        }

        if unaligned_idx != 0 && unaligned_idx + std::mem::size_of::<TestStruct>() <= bytes.len() {
            let misaligned_slice = &bytes[unaligned_idx..(unaligned_idx + std::mem::size_of::<TestStruct>())];
            let view = StructView::<TestStruct>::new(misaligned_slice);
            assert!(view.is_none());
        }
    }
});