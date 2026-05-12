
 use std::sync::atomic::{AtomicUsize, Ordering};
 use crate::{MemMap, RelativePointer};

#[repr(C)]
pub struct VmmHeader {
    pub magic: u64,        
    pub version: u32,
    pub root_offset: u64,    
    pub total_size: u64,
    pub checksum: u64,
}

pub struct PersistentVmm {
    pub memmap: MemMap,
    current_offset: AtomicUsize,
}

impl PersistentVmm {
    pub fn new(memmap: MemMap) -> Self {
    unsafe {

        let header = &mut *(memmap.ptr as *mut VmmHeader);
        
        if header.magic != 0x46544F4F4C53 {
            header.magic = 0x46544F4F4C53;
            header.root_offset = std::mem::size_of::<VmmHeader>() as u64;
        }

        Self {
            memmap,
            current_offset: AtomicUsize::new(std::mem::size_of::<VmmHeader>()),
        }
        }
    }

    pub fn alloc<T>(&self, value: T) -> Option<RelativePointer<T>> {
        let size_t = std::mem::size_of::<T>();
        let align_t = std::mem::align_of::<T>();
        
        loop {
            let current = self.current_offset.load(Ordering::Acquire);
            let aligned = (current + align_t - 1) & !(align_t - 1);
            let next = aligned + size_t;

            if next > self.memmap.len { return None; }

            if self.current_offset.compare_exchange(
                current, next, Ordering::SeqCst, Ordering::Relaxed
            ).is_ok() {
                unsafe {
                    let target_ptr = (self.memmap.ptr as usize + aligned) as *mut T;
                    std::ptr::write(target_ptr, value);
                    return Some(crate::RelativePointer::from_ptr(target_ptr, self.memmap.ptr as usize));
                }
            }
        }
    }

    pub fn get_root_ptr<T>(&self) -> *mut T {
        unsafe {
            let header = &*(self.memmap.ptr as *const VmmHeader);
            (self.memmap.ptr as usize + header.root_offset as usize) as *mut T
        }
    }
    
    pub fn get_root_mut_ref<T>(&self) -> &mut T {
        let root_ptr = self.get_root_ptr::<T>();
        unsafe { &mut *root_ptr }
    }
}