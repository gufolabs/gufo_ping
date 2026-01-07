// ---------------------------------------------------------------------
// Gufo Ping: Linux cBPF utilities
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use socket2::{SockFilter, Socket};
use std::io;

const ICMP_V4_REPLY: u32 = 0;
const ICMP_V6_REPLY: u32 = 129;

#[repr(u16)]
enum Op {
    Ret = 0x06,
    Jne = 0x15,
    Ld = 0x20,
    Ldb = 0x30,
}

#[inline(always)]
fn ld(k: u32) -> SockFilter {
    SockFilter::new(Op::Ld as u16, 0, 0, k)
}

#[inline(always)]
fn ldb(k: u32) -> SockFilter {
    SockFilter::new(Op::Ldb as u16, 0, 0, k)
}

#[inline(always)]
fn jne(offset: u8, k: u32) -> SockFilter {
    SockFilter::new(Op::Jne as u16, 0, offset, k)
}

#[inline(always)]
fn ret(k: u32) -> SockFilter {
    SockFilter::new(Op::Ret as u16, 0, 0, k)
}

#[inline(always)]
pub(super) fn attach_raw4(sock: &Socket, signature: u64) -> io::Result<()> {
    sock.attach_filter(&[
        ldb(0x14),
        jne(5, ICMP_V4_REPLY),
        ld(0x1c),
        jne(3, (signature >> 32) as u32),
        ld(0x20),
        jne(1, (signature & 0xFFFFFFFF) as u32),
        ret(0xffffffff),
        ret(0),
    ])
}

#[inline(always)]
pub(super) fn attach_raw6(sock: &Socket, signature: u64) -> io::Result<()> {
    sock.attach_filter(&[
        ldb(0),
        jne(5, ICMP_V6_REPLY),
        ld(8),
        jne(3, (signature >> 32) as u32),
        ld(0x0c),
        jne(1, (signature & 0xFFFFFFFF) as u32),
        ret(0xffffffff),
        ret(0),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use socket2::SockFilter;

    #[test]
    fn test_ldb() {
        assert_eq!(
            format!("{:?}", ldb(0x14)),
            format!("{:?}", SockFilter::new(0x30, 0, 0, 0x14))
        )
    }

    #[test]
    fn test_ld() {
        assert_eq!(
            format!("{:?}", ld(0x1c)),
            format!("{:?}", SockFilter::new(0x20, 0, 0, 0x1c))
        )
    }

    #[test]
    fn test_jne() {
        assert_eq!(
            format!("{:?}", jne(5, 8)),
            format!("{:?}", SockFilter::new(0x15, 0, 5, 8))
        )
    }

    #[test]
    fn test_ret() {
        assert_eq!(
            format!("{:?}", ret(1)),
            format!("{:?}", SockFilter::new(0x6, 0, 0, 1))
        )
    }
}
