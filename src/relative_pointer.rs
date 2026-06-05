
/*
Position-independent reference utility for cross-process and memory-mapped data structures.

This structure replaces raw absolute hardware memory addresses with a relative byte offset 
calculated from a shared anchor location. It is primarily used to build serializable, graph-like 
structures inside memory-mapped files or shared segments—such as complex nodes in persistent 
indexes or shared-memory rings—ensuring that pointers remain fully functional even when the 
underlying memory buffer is mapped into a completely different virtual address space on subsequent runs.
*/

use std::marker::PhantomData;

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

ftest::test!(relative_pointer_tests, {
    test_null_pointer {
        let rel_ptr = RelativePointer::<i32>::null();
        let base_addr = 0x1000;

        unsafe {
            assert!(rel_ptr.as_ptr(base_addr).is_null());
            assert!(rel_ptr.as_mut_ptr(base_addr).is_null());
        }
    }

    test_from_ptr_and_deref {
        let base_buffer = vec![0i32, 42i32, 0];
        let base_addr = base_buffer.as_ptr() as usize;
        let target_ptr = &base_buffer[1] as *const i32;
        let rel_ptr = RelativePointer::from_ptr(target_ptr, base_addr);
        unsafe {
            let resolved_ptr = rel_ptr.as_ptr(base_addr);
            assert_eq!(resolved_ptr, target_ptr);
            assert_eq!(*resolved_ptr, 42);
        }
    }

    test_mut_ptr_modification {
        let mut value = 100i32;
        let base_addr = 0x2000;

        let target_ptr = &mut value as *mut i32;
        let rel_ptr = RelativePointer::from_ptr(target_ptr, base_addr);

        unsafe {
            let resolved_mut_ptr = rel_ptr.as_mut_ptr(base_addr);
            assert_eq!(resolved_mut_ptr, target_ptr);
            *resolved_mut_ptr = 200;
        }

        assert_eq!(value, 200);
    }
});