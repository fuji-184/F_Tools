use libc::{c_int, c_long, syscall, SYS_get_mempolicy, SYS_mbind};
use std::ptr;
use std::io;

pub struct NumaMemoryBind;

impl NumaMemoryBind {
    pub fn get_current_numa_node() -> io::Result<u32> {
        let mut node: c_int = 0;
        unsafe {
            let res = syscall(
                SYS_get_mempolicy,
                &mut node as *mut c_int,
                ptr::null_mut::<c_long>(),
                0,
                ptr::null_mut::<libc::c_void>(),
                1, 
            );
            if res != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(node as u32)
    }

    pub unsafe fn bind_mem_to_numa_node(ptr: *mut u8, len: usize, node_id: u32) -> io::Result<()> {
        let mut nodemask: c_long = 1 << node_id;
        let res = unsafe { syscall(
            SYS_mbind,
            ptr as *mut libc::c_void,
            len,
            1, 
            &mut nodemask as *mut c_long,
            64, 
            0,
        ) };

        if res != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}