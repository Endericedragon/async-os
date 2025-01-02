use super::{SocketAddr, ToSocketAddrs};

use crate::io::Result;
use aos_api::net::{self as api, AxTcpSocketHandle};
use async_io::{AsyncRead, AsyncWrite};

/// A TCP stream between a local and a remote socket.
pub struct TcpStream(AxTcpSocketHandle);

/// A TCP socket server, listening for connections.
pub struct TcpListener(AxTcpSocketHandle);

impl TcpStream {
    /// Opens a TCP connection to a remote host.
    ///
    /// `addr` is an address of the remote host. Anything which implements
    /// [`ToSocketAddrs`] trait can be supplied for the address; see this trait
    /// documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until a connection is successful. If none of
    /// the addresses result in a successful connection, the error returned from
    /// the last connection attempt (the last address) is returned.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<TcpStream> {
        super::each_addr(addr, async |addr: Result<&SocketAddr>| {
            let addr = addr?;
            let socket = api::ax_tcp_socket();
            api::ax_tcp_connect(&socket, *addr).await?;
            Ok(TcpStream(socket))
        })
        .await
    }

    /// Returns the socket address of the local half of this TCP connection.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        api::ax_tcp_socket_addr(&self.0)
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        api::ax_tcp_peer_addr(&self.0)
    }

    /// Shuts down the connection.
    pub async fn shutdown(&self) -> Result<()> {
        api::ax_tcp_shutdown(&self.0).await
    }
}

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
impl AsyncRead for TcpStream {
    // fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    //     api::ax_tcp_recv(&self.0, buf)
    // }
    fn read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<async_io::Result<usize>> {
        Box::pin(api::ax_tcp_recv(&self.0, buf)).as_mut().poll(cx)
    }
}

impl AsyncWrite for TcpStream {
    // fn write(&mut self, buf: &[u8]) -> Result<usize> {
    //     api::ax_tcp_send(&self.0, buf)
    // }

    // fn flush(&mut self) -> Result<()> {
    //     Ok(())
    // }
    fn write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<async_io::Result<usize>> {
        Box::pin(api::ax_tcp_send(&self.0, buf)).as_mut().poll(cx)
    }

    fn flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<async_io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<async_io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
    /// address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// [`TcpListener::local_addr`] method.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the listener. If
    /// none of the addresses succeed in creating a listener, the error returned
    /// from the last attempt (the last address) is returned.
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<TcpListener> {
        super::each_addr(addr, async |addr: Result<&SocketAddr>| {
            let addr = addr?;
            let backlog = 128;
            let socket = api::ax_tcp_socket();
            api::ax_tcp_bind(&socket, *addr).await?;
            api::ax_tcp_listen(&socket, backlog).await?;
            Ok(TcpListener(socket))
        })
        .await
    }

    /// Returns the local socket address of this listener.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        api::ax_tcp_socket_addr(&self.0)
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, the corresponding [`TcpStream`] and the
    /// remote peer's address will be returned.
    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        api::ax_tcp_accept(&self.0)
            .await
            .map(|(a, b)| (TcpStream(a), b))
    }
}
