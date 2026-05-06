use std::marker::PhantomData;
use std::ops::Deref;

pub struct RelativePointer<T> {
    offset: u64,
    _marker: PhantomData<T>,
}

impl<T> RelativePointer<T> {
    pub fn null() -> Self {
        Self { offset: 0, _marker: PhantomData }
    }

    pub fn from_ptr(ptr: *const T, base_addr: usize) -> Self {
        let offset = (ptr as usize) - base_addr;
        Self { offset: offset as u64, _marker: PhantomData }
    }

    pub unsafe fn as_ptr(&self, base_addr: usize) -> *const T {
        if self.offset == 0 { return std::ptr::null(); }
        (base_addr + self.offset as usize) as *const T
    }

    pub unsafe fn as_mut_ptr(&self, base_addr: usize) -> *mut T {
        if self.offset == 0 { return std::ptr::null_mut(); }
        (base_addr + self.offset as usize) as *mut T
    }
}