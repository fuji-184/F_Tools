
/*
Hardware-aware memory allocation controller to optimize data locality on multi-socket systems.

This structure forces the operating system kernel to allocate specific memory buffers within 
the exact physical RAM bank attached to the CPU core executing the code. It is primarily used 
in high-performance, low-latency infrastructure—such as core database storage engines, heavy 
multithreaded data engines, or low-level network routers—to completely eliminate inter-socket 
bus penalties and maximize hardware cache speeds.
*/

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

ftest::test!(numa_bind_test, {
  // skip because my android doesn't support numa :[
  numa_bind.skip {
    let node = NumaMemoryBind::get_current_numa_node().unwrap();
    println!("Thread berjalan di NUMA Node: {}", node);

    let size = 1024 * 1024 * 1024;
    let layout = std::alloc::Layout::from_size_align(size, 4096).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };

    unsafe {
        NumaMemoryBind::bind_mem_to_numa_node(ptr, size, node).unwrap();
    }
  }
});