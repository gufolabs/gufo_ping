// ---------------------------------------------------------------------
// Gufo Ping: SocketWrapper implementation
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use crate::{Probe, Proto, SelectionPolicy, SessionManager, Timer, slice};
use pyo3::{exceptions::PyOSError, prelude::*};
use socket2::{SockAddr, Socket};
use std::{
    collections::HashMap,
    mem::MaybeUninit,
    net::SocketAddr,
    ops::Not,
    os::unix::io::AsRawFd,
    sync::atomic::{AtomicU16, Ordering},
};
use twox_hash::XxHash64;

const MAX_SIZE: usize = 4096;
const REQUEST_TIMEOUT: u64 = 0;

/// Python class wrapping socket implementation
#[pyclass]
pub(crate) struct SocketWrapper {
    proto: &'static Proto,
    io: Socket,
    timer: Timer,
    signature: u64,
    seq: AtomicU16,
    timeout: u64,
    sessions: SessionManager,
    buf: [MaybeUninit<u8>; MAX_SIZE],
}

#[pymethods]
impl SocketWrapper {
    /// Python constructor
    #[new]
    fn new(policy_id: u8, timeout_ns: u64, coarse: bool) -> PyResult<Self> {
        let policy = SelectionPolicy::try_from(policy_id)?;
        let proto = <&'static Proto>::try_from(policy)?;
        // Create socket
        let (io, signature) = proto.create_socket()?;
        Ok(Self {
            proto,
            io,
            timer: Timer::new(coarse),
            signature,
            seq: AtomicU16::new(Self::scramble_to_u16(signature)),
            sessions: SessionManager::new(),
            timeout: timeout_ns,
            buf: slice::get_buffer_mut(),
        })
    }

    fn bind(&mut self, addr: &str) -> PyResult<()> {
        let src_addr = self.proto.to_sockaddr(addr)?;
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
    fn clean_ip(&self, addr: &str) -> PyResult<String> {
        Ok(self.proto.to_ip(addr)?)
    }
    /// Send single ICMP echo request
    fn send(&mut self, addr: &str, size: usize) -> PyResult<u64> {
        // Parse IP address
        let to_addr = self.proto.to_sockaddr(addr)?;
        // Get timestamp
        let ts = self.timer.get_ts();
        let seq = self.next_seq();
        let probe = Probe::new(seq, self.signature, ts);
        let request_id = probe.get_request_id();
        let buf = self.proto.encode_request(probe, &mut self.buf, size);
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
        let ts = self.timer.get_ts();
        // Rewrite when recvmmsg function will be available.
        while let Ok((size, addr)) = self.io.recv_from(&mut self.buf) {
            let buf = slice::slice_assume_init_ref(&self.buf[..size]);
            if let Some(pkt) = self.proto.decode_reply(buf)
                && pkt.get_signature() == self.signature
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
    // Scramble u64 to u16
    fn scramble_to_u16(v: u64) -> u16 {
        let s1 = (v >> 32) as u32 ^ v as u32;
        (s1 >> 16) as u16 ^ s1 as u16
    }

    // Generate next sequence number
    #[inline(always)]
    fn next_seq(&self) -> u16 {
        self.seq.fetch_add(1, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scramble_to_u16() {
        let v = [(0u64, 0u16), (0xffffffffffffffff, 0), (1, 1)];
        for (r, expected) in v.iter() {
            assert_eq!(SocketWrapper::scramble_to_u16(*r), *expected);
        }
    }
}
