
/*
High-performance zero-copy data transfer utility for kernel-level I/O.

This structure interfaces directly with Linux kernel primitives—specifically `splice` and `sendfile`—to move data between file descriptors without copying it through user-space memory. By bypassing the traditional read-to-buffer and buffer-to-write cycle, it eliminates expensive context switches and memory duplication. It is primarily used to build ultra-fast networking and storage services—such as high-throughput static web servers or file-streaming proxies—where minimizing CPU overhead and maximizing throughput for large data transfers are paramount.
*/

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

ftest::test!(zero_copy_tests, {
    test_zero_copy_splice_pipe {
        use std::fs::File;
        use std::io::{Read, Write};

        let dir = std::env::temp_dir();
        let src_path = dir.join("zero_copy_src_splice.bin");
        let dst_path = dir.join("zero_copy_dst_splice.bin");

        {
            let mut src_file = File::create(&src_path).unwrap();
            src_file.write_all(b"zero-copy-splice-data").unwrap();
        }

        {
            let src_file = File::open(&src_path).unwrap();
            let dst_file = File::create(&dst_path).unwrap();

            let bytes_sent = ZeroCopy::send(&src_file, &dst_file, 21);
            if let Err(err) = bytes_sent {
                assert!(
                    err.kind() == std::io::ErrorKind::PermissionDenied || 
                    err.raw_os_error() == Some(libc::EINVAL)
                );
            } else {
                assert_eq!(bytes_sent.unwrap(), 21);
                
                let mut check_file = File::open(&dst_path).unwrap();
                let mut buf = Vec::new();
                check_file.read_to_end(&mut buf).unwrap();
                assert_eq!(buf, b"zero-copy-splice-data");
            }
        }

        let _ = std::fs::remove_file(src_path);
        let _ = std::fs::remove_file(dst_path);
    }

    test_zero_copy_sendfile {
        use std::fs::File;
        use std::io::{Read, Write};
        use std::os::unix::net::UnixStream;

        let dir = std::env::temp_dir();
        let src_path = dir.join("zero_copy_src_sendfile.bin");

        {
            let mut src_file = File::create(&src_path).unwrap();
            src_file.write_all(b"zero-copy-sendfile-data").unwrap();
        }

        {
            let (mut rx, tx) = UnixStream::pair().unwrap();
            let src_file = File::open(&src_path).unwrap();

            let bytes_sent = ZeroCopy::send_file(&src_file, &tx, 23);
            if let Err(err) = bytes_sent {
                assert!(
                    err.kind() == std::io::ErrorKind::PermissionDenied || 
                    err.raw_os_error() == Some(libc::EINVAL) ||
                    err.raw_os_error() == Some(libc::ENOTSOCK)
                );
            } else {
                assert_eq!(bytes_sent.unwrap(), 23);

                let mut buf = vec![0u8; 23];
                rx.read_exact(&mut buf).unwrap();
                assert_eq!(buf, b"zero-copy-sendfile-data");
            }
        }

        let _ = std::fs::remove_file(src_path);
    }
});