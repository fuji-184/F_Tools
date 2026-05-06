use std::os::unix::io::AsRawFd;
use std::io;
use libc::{splice, pipe, close, SPLICE_F_MOVE, SPLICE_F_MORE};

pub struct ZeroCopy;

impl ZeroCopy {
    pub fn send<R: AsRawFd, W: AsRawFd>(reader: &R, writer: &W, len: usize) -> io::Result<usize> {
        let mut fds = [0i32; 2];
        unsafe {
            if pipe(fds.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        let [p_rx, p_tx] = fds;

        let result = (|| unsafe {

            let s1 = splice(reader.as_raw_fd(), std::ptr::null_mut(), p_tx, std::ptr::null_mut(), len, SPLICE_F_MOVE | SPLICE_F_MORE);
            if s1 < 0 { return Err(io::Error::last_os_error()); }


            let s2 = splice(p_rx, std::ptr::null_mut(), writer.as_raw_fd(), std::ptr::null_mut(), s1 as usize, SPLICE_F_MOVE | SPLICE_F_MORE);
            if s2 < 0 { return Err(io::Error::last_os_error()); }

            Ok(s2 as usize)
        })();

        unsafe {
            close(p_rx);
            close(p_tx);
        }

        result
    }

    pub fn send_file<F: AsRawFd, S: AsRawFd>(file: &F, socket: &S, len: usize) -> io::Result<usize> {
        let res = unsafe {
            libc::sendfile(socket.as_raw_fd(), file.as_raw_fd(), std::ptr::null_mut(), len)
        };
        
        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res as usize)
        }
    }
}