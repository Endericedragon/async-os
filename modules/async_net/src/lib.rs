//! [ArceOS](https://github.com/rcore-os/arceos) network module.
//!
//! It provides unified networking primitives for TCP/UDP communication
//! using various underlying network stacks. Currently, only [smoltcp] is
//! supported.
//!
//! # Organization
//!
//! - [`TcpSocket`]: A TCP socket that provides POSIX-like APIs.
//! - [`UdpSocket`]: A UDP socket that provides POSIX-like APIs.
//! - [`dns_query`]: Function for DNS query.
//!
//! # Cargo Features
//!
//! - `smoltcp`: Use [smoltcp] as the underlying network stack. This is enabled
//!   by default.
//!
//! [smoltcp]: https://github.com/smoltcp-rs/smoltcp

#![no_std]
#![feature(async_closure)]

#[macro_use]
extern crate log;
extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "smoltcp")] {
        mod smoltcp_impl;
        use smoltcp_impl as net_impl;
    }
}

pub use self::net_impl::TcpSocket;
pub use self::net_impl::UdpSocket;
pub use self::net_impl::{
    add_membership, dns_query, from_core_sockaddr, into_core_sockaddr, poll_interfaces,
};
pub use self::net_impl::{bench_receive, bench_transmit};
pub use smoltcp::time::Duration;
pub use smoltcp::wire::{
    IpAddress as IpAddr, IpEndpoint, Ipv4Address as Ipv4Addr, Ipv6Address as Ipv6Addr,
};
pub use smoltcp_impl::NetlinkSocket;

mod ctypes;
pub use ctypes::sockaddr_nl as NetlinkEndpoint;

#[derive(Debug, Clone, Copy)]
pub enum SocketAddr {
    IpPortPair(IpEndpoint),
    // NetLink方面，参考了https://github.com/rust-netlink/netlink-sys/blob/main/src/addr.rs的设计
    NetlinkEndpoint(NetlinkEndpoint),
}

impl SocketAddr {
    pub fn new_ip_port_pair(ip_port_pair: IpEndpoint) -> Self {
        Self::IpPortPair(ip_port_pair)
    }

    pub fn new_netlink_endpoint(netlink_endpoint: NetlinkEndpoint) -> Self {
        Self::NetlinkEndpoint(netlink_endpoint)
    }

    pub fn default_netlink_endpoint() -> Self {
        let addr: NetlinkEndpoint = unsafe { core::mem::zeroed() };
        Self::NetlinkEndpoint(addr)
    }
}

use axdriver::{prelude::*, AxDeviceContainer};

/// Initializes the network subsystem by NIC devices.
pub async fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    let dev = net_devs.take_one().expect("No NIC device found!");
    info!("  use NIC 0: {:?}", dev.device_name());
    net_impl::init(dev).await;
}
