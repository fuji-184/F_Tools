use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
use std::io;
//use std::thread::ThreadId;

pub struct ThreadToCPU;

impl ThreadToCPU {
    pub fn pin_current_thread(core_id: usize) -> io::Result<()> {
        unsafe {
            let mut cpuset: cpu_set_t = std::mem::zeroed();
            CPU_ZERO(&mut cpuset);
            CPU_SET(core_id, &mut cpuset);

            let res = sched_setaffinity(
                0, 
                std::mem::size_of::<cpu_set_t>(),
                &cpuset,
            );

            if res != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub fn set_realtime_priority() -> io::Result<()> {
        unsafe {
            let param = libc::sched_param { sched_priority: 99 };
            let res = libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
            
            if res != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

ftest::test!(thread_to_cpu_tests, {
    test_pin_current_thread {
        let res = ThreadToCPU::pin_current_thread(0);
        if res.is_err() {
            let err = res.unwrap_err();
            assert!(
                err.kind() == std::io::ErrorKind::PermissionDenied || 
                err.raw_os_error() == Some(libc::EINVAL)
            );
        } else {
            assert!(res.is_ok());
        }
    }

    test_set_realtime_priority {
        let res = ThreadToCPU::set_realtime_priority();
        if res.is_err() {
            let err = res.unwrap_err();
            assert!(
                err.kind() == std::io::ErrorKind::PermissionDenied || 
                err.raw_os_error() == Some(libc::EPERM)
            );
        } else {
            assert!(res.is_ok());
        }
    }
});