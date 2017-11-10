use mio::Poll;
use net2::TcpBuilder;
use net2::TcpStreamExt;
use std::io::Read;
use std::io::Write;
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net;
use std::os::unix::io::AsRawFd;
use std::result::Result;

use stream::utils::*;

/// A buffered TCP stream that registers itself for readiness events when the buffer is not empty.
/// Use `Write` instance for writing to the buffer. Use `Read` instance when the stream is ready
/// for reading. Use `send` when the stream is ready for writing.
pub struct TcpStream<'poll> {
    inner: net::TcpStream,
    poll: &'poll Poll,
    out_buf: Vec<u8>,
}

#[derive(Debug)]
pub enum TcpError {
    IoError(io::Error),
    CantResolveAddr,
}

impl<'poll> TcpStream<'poll> {
    pub fn new(
        poll: &'poll Poll,
        serv_addr: &str,
        serv_port: u16,
    ) -> Result<TcpStream<'poll>, TcpError> {
        let mut addr_iter = (serv_addr, serv_port)
            .to_socket_addrs()
            .map_err(TcpError::IoError)?;
        let addr = addr_iter.next().ok_or(TcpError::CantResolveAddr)?;
        let stream = {
            match addr {
                SocketAddr::V4(_) =>
                    TcpBuilder::new_v4().unwrap().to_tcp_stream().unwrap(),
                SocketAddr::V6(_) =>
                    TcpBuilder::new_v6().unwrap().to_tcp_stream().unwrap(),
            }
        };
        stream.set_nonblocking(true).unwrap();
        // This will fail with EINPROGRESS
        let _ = stream.connect(addr);
        register_for_r(poll, stream.as_raw_fd());
        Ok(TcpStream {
            inner: stream,
            poll: poll,
            out_buf: Vec::with_capacity(1024),
        })
    }

    /// Call when the stream is ready for writing.
    pub fn write_ready(&mut self) -> io::Result<()> {
        let to_send = self.out_buf.len();
        match self.inner.write(&self.out_buf) {
            Ok(bytes_sent) => {
                self.out_buf.drain(0 .. bytes_sent);
                let register =
                    if bytes_sent == to_send {
                        reregister_for_r
                    } else {
                        reregister_for_rw
                    };
                register(&self.poll, self.inner.as_raw_fd());
                Ok(())
            }
            Err(err) => {
                reregister_for_rw(&self.poll, self.inner.as_raw_fd());
                if err.kind() == io::ErrorKind::WouldBlock {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }
}

// Drop just deregisters the socket
impl<'poll> Drop for TcpStream<'poll> {
    fn drop(&mut self) {
        deregister(&self.poll, self.inner.as_raw_fd());
    }
}

impl<'poll> Read for TcpStream<'poll> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<'poll> Write for TcpStream<'poll> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // TODO inefficient when the socket is already ready for writing
        self.out_buf.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write_ready()
    }
}
