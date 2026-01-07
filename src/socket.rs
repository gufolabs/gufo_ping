// ---------------------------------------------------------------------
// Gufo Ping: SocketWrapper implementation
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use crate::{IcmpPacket, SessionManager, filter::Filter};
use coarsetime::Clock;
use pyo3::{
    exceptions::{PyOSError, PyValueError},
    prelude::*,
};
use rand::Rng;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    collections::HashMap,
    convert::TryFrom,
    mem::MaybeUninit,
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
    ops::Not,
    os::unix::io::AsRawFd,
    time::Instant,
};
use twox_hash::XxHash64;

const MAX_SIZE: usize = 4096;
const ICMP_SIZE: usize = 8;
const REQUEST_TIMEOUT: u64 = 0;

// @todo: Is necessary?
enum Afi {
    IPV4,
    IPV6,
}

struct Proto {
    afi: Afi,
    domain: Domain,
    protocol: Protocol,
    filter: Filter,
    ip_header_size: usize,
    icmp_request_type: u8,
    icmp_reply_type: u8,
}

static IPV4: Proto = Proto {
    afi: Afi::IPV4,
    domain: Domain::IPV4,
    protocol: Protocol::ICMPV4,
    #[cfg(target_os = "linux")]
    filter: Filter::LinuxRaw4,
    #[cfg(not(target_os = "linux"))]
    filter: Filter::None,
    ip_header_size: 20,
    icmp_request_type: 8,
    icmp_reply_type: 0,
};

static IPV6: Proto = Proto {
    afi: Afi::IPV6,
    domain: Domain::IPV6,
    protocol: Protocol::ICMPV6,
    #[cfg(target_os = "linux")]
    filter: Filter::LinuxRaw6,
    #[cfg(not(target_os = "linux"))]
    filter: Filter::None,
    ip_header_size: 0, // No IPv6 header is passed over socket
    icmp_request_type: 128,
    icmp_reply_type: 129,
};

/// Python class wrapping socket implementation
#[pyclass]
pub(crate) struct SocketWrapper {
    proto: &'static Proto,
    io: Socket,
    signature: u64,
    timeout: u64,
    sessions: SessionManager,
    start: Instant,
    coarse: bool,
    buf: [MaybeUninit<u8>; MAX_SIZE],
}

#[pymethods]
impl SocketWrapper {
    /// Python constructor
    #[new]
    fn new(afi: u8, timeout_ns: u64, coarse: bool) -> PyResult<Self> {
        let proto = match afi {
            4 => &IPV4,
            6 => &IPV6,
            _ => return Err(PyValueError::new_err("invalid afi".to_string())),
        };
        // Generate socket signature
        let signature = rand::rng().random();
        // Create socket for given address family
        let io = Socket::new(proto.domain, Type::RAW, Some(proto.protocol))
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        proto.filter.attach_filter(&io, signature)?;
        // Mark socket as non-blocking
        io.set_nonblocking(true)
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        Ok(Self {
            proto,
            io,
            signature,
            sessions: SessionManager::new(),
            timeout: timeout_ns,
            start: Instant::now(),
            coarse,
            buf: unsafe { MaybeUninit::uninit().assume_init() },
        })
    }

    fn bind(&mut self, addr: &str) -> PyResult<()> {
        let src_addr: SockAddr = match self.proto.afi {
            Afi::IPV4 => SocketAddrV4::new(addr.parse()?, 0).into(),
            Afi::IPV6 => SocketAddrV6::new(addr.parse()?, 0, 0, 0).into(),
        };
        self.io.bind(&src_addr)?;
        Ok(())
    }
    /// Set default outgoing packets' TTL
    fn set_ttl(&self, ttl: u32) -> PyResult<()> {
        self.io.set_ttl_v4(ttl)?;
        Ok(())
    }
    /// Set IPv6 unicast hops
    fn set_unicast_hops(&self, ttl: u32) -> PyResult<()> {
        self.io.set_unicast_hops_v6(ttl)?;
        Ok(())
    }
    /// Set default outgoing packets' ToS
    fn set_tos(&self, tos: u32) -> PyResult<()> {
        self.io.set_tos_v4(tos)?;
        Ok(())
    }
    /// Set default outgoung packet's IPv6 TCLASS
    fn set_tclass(&self, tclass: u32) -> PyResult<()> {
        self.io.set_tclass_v6(tclass)?;
        Ok(())
    }

    /// Set internal socket's send buffer size
    fn set_send_buffer_size(&self, size: usize) -> PyResult<()> {
        // @todo: get wmem_max limit on Linux
        let mut effective_size = size;
        while effective_size > 0 {
            if self.io.set_send_buffer_size(effective_size).is_ok() {
                return Ok(());
            }
            effective_size >>= 1;
        }
        Err(PyOSError::new_err("unable to set buffer size"))
    }

    /// Set internal socket's receive buffer size
    fn set_recv_buffer_size(&self, size: usize) -> PyResult<()> {
        let mut effective_size = size;
        while effective_size > 0 {
            if self.io.set_recv_buffer_size(effective_size).is_ok() {
                return Ok(());
            }
            effective_size >>= 1;
        }
        Err(PyOSError::new_err("unable to set buffer size"))
    }

    /// Get socket's file descriptor
    fn get_fd(&self) -> PyResult<i32> {
        Ok(self.io.as_raw_fd())
    }

    /// Normalize address
    fn clean_ip(&self, addr: String) -> PyResult<String> {
        Ok(match self.proto.afi {
            Afi::IPV4 => SocketAddrV4::new(addr.parse()?, 0).ip().to_string(),
            Afi::IPV6 => SocketAddrV6::new(addr.parse()?, 0, 0, 0).ip().to_string(),
        })
    }
    /// Send single ICMP echo request
    fn send(&mut self, addr: String, request_id: u16, seq: u16, size: usize) -> PyResult<u64> {
        // Parse IP address
        let to_addr: SockAddr = match self.proto.afi {
            Afi::IPV4 => SocketAddrV4::new(addr.parse()?, 0).into(),
            Afi::IPV6 => SocketAddrV6::new(addr.parse()?, 0, 0, 0).into(),
        };
        // Get timestamp
        let ts = self.get_ts();
        let pkt = IcmpPacket::new(
            self.proto.icmp_request_type,
            request_id,
            seq,
            self.signature,
            ts,
            size - self.proto.ip_header_size,
        );
        let n = pkt.write(&mut self.buf);
        let buf = Self::slice_assume_init_ref(&self.buf[..n]);
        self.io
            .send_to(buf, &to_addr)
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        let sid = self.get_sid(&to_addr, request_id, seq);
        self.sessions.register(sid, ts + self.timeout);
        Ok(sid)
    }

    /// Receive all pending icmp echo replies.
    /// Returns dict of <session id> -> rtt
    fn recv(&mut self) -> PyResult<Option<HashMap<u64, u64>>> {
        let mut r = HashMap::<u64, u64>::new();
        let ts = self.get_ts();
        // Rewrite when recvmmsg function will be available.
        while let Ok((size, addr)) = self.io.recv_from(&mut self.buf) {
            // Drop too short packets
            if size < self.proto.ip_header_size + ICMP_SIZE {
                continue;
            }
            let buf = Self::slice_assume_init_ref(&self.buf[self.proto.ip_header_size..size]);
            // Parse packet
            if let Ok(pkt) = IcmpPacket::try_from(buf)
                && pkt.is_match(self.proto.icmp_reply_type, self.signature)
            {
                // Measure RTT
                let pkt_ts = pkt.get_ts();
                let delay = if ts > pkt_ts {
                    ts - pkt_ts
                } else {
                    1 // Minimal delay
                };
                let sid = self.get_sid(&addr, pkt.get_request_id(), pkt.get_seq());
                r.insert(sid, delay);
                self.sessions.remove(sid, pkt_ts + self.timeout);
            }
        }
        // Check for expired sessions
        for sid in self.sessions.drain_expired(ts) {
            r.insert(sid, REQUEST_TIMEOUT);
        }
        Ok(r.is_empty().not().then_some(r))
    }
}

impl SocketWrapper {
    /// Get current timestamp.
    /// Use CLOCK_MONOTONIC by default.
    /// Switch to CLOCK_MONOTONIC_COARSE when .set_coarse(true)
    pub fn get_ts(&self) -> u64 {
        if self.coarse {
            // CLOCK_MONOTONIC_COARSE
            Clock::now_since_epoch().as_nanos()
        } else {
            // CLOCK_MONOTONIC
            self.start.elapsed().as_nanos() as u64
        }
    }

    /// Generate session id
    fn get_sid(&self, addr: &SockAddr, request_id: u16, seq: u16) -> u64 {
        match addr.as_socket() {
            Some(a) => match a {
                SocketAddr::V4(x) => {
                    ((request_id as u64) << 48) | ((seq as u64) << 32) | (x.ip().to_bits() as u64)
                }
                SocketAddr::V6(x) => XxHash64::oneshot(
                    ((request_id as u64) << 16) | (seq as u64),
                    x.ip().octets().as_slice(),
                ),
            },
            None => 0,
        }
    }

    // Assume buffer initialized
    // @todo: Replace with BufRead.filled()
    // @todo: Replace when `maybe_uninit_slice` feature
    // will be stabilized
    const fn slice_assume_init_ref(slice: &[MaybeUninit<u8>]) -> &[u8] {
        //MaybeUninit::slice_assume_init_ref(&self.buf[self.proto.ip_header_size..size]);
        unsafe { &*(slice as *const [MaybeUninit<u8>] as *const [u8]) }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_ipv4_sid() {
//         let sock = SocketWrapper::new(4).unwrap();
//         let addr = SocketAddrV4::new("127.0.0.1".parse().unwrap(), 0);
//         let sid = sock.get_sid(&addr.into(), 0x102, 1);
//         assert_eq!(sid, 1);
//     }
// }
