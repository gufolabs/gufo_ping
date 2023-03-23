// ---------------------------------------------------------------------
// Gufo Ping: SocketWrapper implementation
// ---------------------------------------------------------------------
// Copyright (C) 2022-23, Gufo Labs
// ---------------------------------------------------------------------

use super::{IcmpPacket, Session};
use coarsetime::Clock;
use pyo3::{
    exceptions::{PyOSError, PyValueError},
    prelude::*,
};
use rand::Rng;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::collections::{BTreeSet, HashMap};
use std::convert::TryFrom;
use std::mem::MaybeUninit;
use std::net::{SocketAddrV4, SocketAddrV6};
use std::os::unix::io::AsRawFd;
use std::time::Instant;

const MAX_SIZE: usize = 4096;
const ICMP_SIZE: usize = 8;

enum Afi {
    IPV4,
    IPV6,
}

struct Proto {
    afi: Afi,
    domain: Domain,
    protocol: Protocol,
    ip_header_size: usize,
    icmp_request_type: u8,
    icmp_reply_type: u8,
}

static IPV4: Proto = Proto {
    afi: Afi::IPV4,
    domain: Domain::IPV4,
    protocol: Protocol::ICMPV4,
    ip_header_size: 20,
    icmp_request_type: 8,
    icmp_reply_type: 0,
};

static IPV6: Proto = Proto {
    afi: Afi::IPV6,
    domain: Domain::IPV6,
    protocol: Protocol::ICMPV6,
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
    sessions: BTreeSet<Session>,
    start: Instant,
    coarse: bool,
    buf: [MaybeUninit<u8>; MAX_SIZE],
}

#[pymethods]
impl SocketWrapper {
    /// Python constructor
    #[new]
    fn new(afi: u8) -> PyResult<Self> {
        let proto = match afi {
            4 => &IPV4,
            6 => &IPV6,
            _ => return Err(PyValueError::new_err("invalid afi".to_string())),
        };
        // Create socket for given address family
        let io = Socket::new(proto.domain, Type::RAW, Some(proto.protocol))
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        // Mark socket as non-blocking
        io.set_nonblocking(true)
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        let mut rng = rand::thread_rng();
        Ok(Self {
            proto,
            io,
            signature: rng.gen(),
            sessions: BTreeSet::new(),
            timeout: 1_000_000_000,
            start: Instant::now(),
            coarse: false,
            buf: unsafe { MaybeUninit::uninit().assume_init() },
        })
    }

    ///
    fn bind(&mut self, addr: &str) -> PyResult<()> {
        let src_addr: SockAddr = match self.proto.afi {
            Afi::IPV4 => SocketAddrV4::new(addr.parse()?, 0).into(),
            Afi::IPV6 => SocketAddrV6::new(addr.parse()?, 0, 0, 0).into(),
        };
        self.io.bind(&src_addr)?;
        Ok(())
    }
    /// Set default timeout, in nanoseconds
    fn set_timeout(&mut self, timeout: u64) -> PyResult<()> {
        self.timeout = timeout;
        Ok(())
    }

    /// Set default outgoing packets' TTL
    fn set_ttl(&self, ttl: u32) -> PyResult<()> {
        self.io.set_ttl(ttl)?;
        Ok(())
    }

    /// Set default outgoing packets' ToS
    fn set_tos(&self, tos: u32) -> PyResult<()> {
        self.io.set_tos(tos)?;
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

    /// Switch to CLOCK_MONOTONIC_COARSE implementation
    fn set_coarse(&mut self, ct: bool) -> PyResult<()> {
        self.coarse = ct;
        Ok(())
    }

    /// Enable accelerated socket processing
    fn set_accelerated(&self, a: bool) -> PyResult<()> {
        if a {
            self.enable_accelerated()?
        } else {
            self.disable_accelerated()?
        }
        Ok(())
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
    fn send(&mut self, addr: String, request_id: u16, seq: u16, size: usize) -> PyResult<()> {
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
        let buf = unsafe { Self::slice_assume_init_ref(&self.buf[..n]) };
        self.io
            .send_to(buf, &to_addr)
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        self.sessions
            .insert(Session::new(&pkt.get_sid(addr), ts + self.timeout));
        Ok(())
    }

    /// Receive all pending icmp echo replies.
    /// Returns dict of <session id> -> rtt
    fn recv(&mut self) -> PyResult<Option<HashMap<String, u64>>> {
        let mut r = HashMap::<String, u64>::new();
        while let Ok((size, addr)) = self.io.recv_from(&mut self.buf) {
            // Drop too short packets
            if size < self.proto.ip_header_size + ICMP_SIZE {
                continue;
            }
            let buf =
                unsafe { Self::slice_assume_init_ref(&self.buf[self.proto.ip_header_size..size]) };
            // Parse packet
            if let Ok(pkt) = IcmpPacket::try_from(buf) {
                if pkt.is_match(self.proto.icmp_reply_type, self.signature) {
                    // Measure RTT
                    let ts = self.get_ts();
                    let pkt_ts = pkt.get_ts();
                    let delay = if ts > pkt_ts {
                        ts - pkt_ts
                    } else {
                        1 // Minimal delay
                    };
                    // Convert SockAddr to printable form
                    let paddr = match self.proto.afi {
                        Afi::IPV4 => addr.as_socket_ipv4().unwrap().ip().to_string(),
                        Afi::IPV6 => addr.as_socket_ipv6().unwrap().ip().to_string(),
                    };
                    r.insert(pkt.get_sid(paddr.clone()), delay);
                    self.sessions
                        .remove(&Session::new(&pkt.get_sid(paddr), pkt_ts + self.timeout));
                }
            }
        }
        if !r.is_empty() {
            Ok(Some(r))
        } else {
            Ok(None)
        }
    }

    /// Get list of session ids of expired sessions
    fn get_expired(&mut self) -> PyResult<Option<Vec<String>>> {
        let mut r = Vec::<Session>::new();
        // @todo: Waiting until map_first_last API
        let ts = self.get_ts();
        // Extract and cleanup expired sessions
        // NOTE:
        // We cannnot remove items while holding the iterator.
        // Remove them later in separate cycle.
        // To be replaced with `pop_first` after the `map_first_last`
        // feeature will be stabilized.
        for item in self.sessions.iter() {
            if !item.is_expired(ts) {
                break;
            }
            r.push(item.clone());
        }
        // Cleanup expired sessions sessions
        for item in r.iter() {
            self.sessions.remove(item);
        }
        //  Return result
        if r.is_empty() {
            Ok(None)
        } else {
            Ok(Some(r.iter().map(|x| x.get_sid()).collect()))
        }
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

    /// Attach cBPF filter to socket to reduce context switches
    #[cfg(target_os = "linux")]
    fn enable_accelerated(&self) -> std::io::Result<()> {
        #[inline]
        fn op(code: u16, jt: u8, jf: u8, k: u32) -> sock_filter {
            sock_filter { code, jt, jf, k }
        }

        use libc::sock_filter;

        match self.proto.afi {
            Afi::IPV4 => {
                let filters = [
                    op(0x30, 0, 0, 0x00000014),                           // ldb [20]
                    op(0x15, 0, 5, self.proto.icmp_reply_type as u32),    // jne #0x0, drop
                    op(0x20, 0, 0, 0x0000001c),                           // ld [28]
                    op(0x15, 0, 3, (self.signature >> 32) as u32),        // jne #sig1, drop
                    op(0x20, 0, 0, 0x00000020),                           // ld [32]
                    op(0x15, 0, 1, (self.signature & 0xFFFFFFFF) as u32), // jne #sig2, drop
                    op(0x06, 0, 0, 0xffffffff),                           // ret #-1
                    op(0x06, 0, 0, 0000000000),                           // drop: ret #0
                ];
                self.io.attach_filter(&filters)?;
            }
            Afi::IPV6 => {
                let filters = [
                    op(0x30, 0, 0, 0x00000000),                           // ldb [0]
                    op(0x15, 0, 5, self.proto.icmp_reply_type as u32),    // jne #0x81, drop
                    op(0x20, 0, 0, 0x00000008),                           // ld [8]
                    op(0x15, 0, 3, (self.signature >> 32) as u32),        // jne #sig1, drop
                    op(0x20, 0, 0, 0x0000000c),                           // ld [12]
                    op(0x15, 0, 1, (self.signature & 0xFFFFFFFF) as u32), // jne #sig2, drop
                    op(0x06, 0, 0, 0xffffffff),                           // ret #-1
                    op(0x06, 0, 0, 0000000000),                           // drop: ret #0
                ];

                self.io.attach_filter(&filters)?;
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn enable_accelerated(&self) -> std::io::Result<()> {
        Ok(())
    }

    /// Remove BPF filter from socket
    #[cfg(target_os = "linux")]
    fn disable_accelerated(&self) -> std::io::Result<()> {
        self.io.detach_filter()?;
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn disable_accelerated(&self) -> std::io::Result<()> {
        Ok(())
    }
    // Assume buffer initialized
    // @todo: Replace with BufRead.filled()
    // @todo: Replace when `maybe_uninit_slice` feature
    // will be stabilized
    const unsafe fn slice_assume_init_ref(slice: &[MaybeUninit<u8>]) -> &[u8] {
        //MaybeUninit::slice_assume_init_ref(&self.buf[self.proto.ip_header_size..size]);
        &*(slice as *const [MaybeUninit<u8>] as *const [u8])
    }
}
