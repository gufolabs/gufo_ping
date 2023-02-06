// ---------------------------------------------------------------------
// Gufo Ping: ICMP packet constructing and parsing
// ---------------------------------------------------------------------
// Copyright (C) 2022-23, Gufo Labs
// ---------------------------------------------------------------------

use byteorder::{BigEndian, ByteOrder};
use internet_checksum::checksum;
use std::convert::TryFrom;
use std::mem::MaybeUninit;

/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// | Type          | Code          | Checksum                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// | Request Id                    | Sequence                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// Payload
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// | Signature                                                     |
/// |                                                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// | Timestamp                                                     |
/// |                                                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// Where:
/// * `type`
///   * `8`: echo request (ICMPv4)
///   * `0`: echo reply (ICMPv4)
///   * `128`: echo request (ICMPv6)
///   * `129`: echo reply (ICMPv6)
/// * `code` - 0
/// ```

#[derive(Debug, PartialEq)]
pub(crate) struct IcmpPacket {
    icmp_type: u8,
    request_id: u16,
    seq: u16,
    signature: u64,
    ts: u64,
    size: usize, // ip payload size, without IP header
}

impl IcmpPacket {
    pub fn new(
        icmp_type: u8,
        request_id: u16,
        seq: u16,
        signature: u64,
        ts: u64,
        size: usize,
    ) -> Self {
        IcmpPacket {
            icmp_type,
            request_id,
            seq,
            signature,
            ts,
            size,
        }
    }

    pub fn get_sid(&self, addr: String) -> String {
        format!("{}-{}-{}", addr, self.request_id, self.seq)
    }

    pub fn get_ts(&self) -> u64 {
        self.ts
    }

    pub fn is_match(&self, icmp_type: u8, sig: u64) -> bool {
        self.icmp_type == icmp_type && self.signature == sig
    }

    // @todo: Replace with MaybeUninit::slice_assume_init_mut
    // when `maybe_uninit_slice` feature will be stabilized
    #[inline(always)]
    unsafe fn slice_assume_init_mut(slice: &mut [MaybeUninit<u8>]) -> &mut [u8] {
        &mut *(slice as *mut [MaybeUninit<u8>] as *mut [u8])
    }

    /// Write packet to buffer
    pub fn write(&self, buf: &mut [MaybeUninit<u8>]) -> usize {
        //
        // Assume buffer initialized
        let buf = unsafe { Self::slice_assume_init_mut(&mut buf[..self.size]) };
        // Write type, fill code and checksum with 0
        BigEndian::write_u32(buf, (self.icmp_type as u32) << 24);
        // Request id, 2 octets
        BigEndian::write_u16(&mut buf[4..], self.request_id);
        // Sequence, 2 octets
        BigEndian::write_u16(&mut buf[6..], self.seq);
        // Signature, 8 octets
        BigEndian::write_u64(&mut buf[8..], self.signature);
        // Timestamp, 8 octets
        BigEndian::write_u64(&mut buf[16..], self.ts);
        // Generate padding, Fill rest by "A"
        if self.size > 24 {
            buf[24..].fill(48u8);
        }
        // Calculate checksum
        // RFC-1071
        let cs = checksum(buf);
        buf[2] = cs[0];
        buf[3] = cs[1];
        self.size
    }
}

// Parse IcmpPacket
impl TryFrom<&[u8]> for IcmpPacket {
    type Error = &'static str;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.len() < 24 {
            return Err("too short");
        }
        let size = buf.len();
        Ok(Self {
            icmp_type: buf[0],
            request_id: BigEndian::read_u16(&buf[4..]),
            seq: BigEndian::read_u16(&buf[6..]),
            signature: BigEndian::read_u64(&buf[8..]),
            ts: BigEndian::read_u64(&buf[16..]),
            size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    static ICMPV4_REQ: &[u8] = &[
        8, 0, 213, 217, // Type, Code, Checksum
        1, 2, 0, 1, // Request id, Sequence
        0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
        0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, // Padding
    ];

    static ICMPV4_REQ_PKT: IcmpPacket = IcmpPacket {
        icmp_type: 8,
        request_id: 0x0102,
        seq: 1,
        signature: 0xdeadbeefdeadbeef,
        ts: 0x01020304,
        size: 64 - 20,
    };

    static ICMPV4_REPLY: &[u8] = &[
        0, 0, 28, 137, // Type, Code, Checksum (faked)
        1, 2, 0, 1, // Request id, sequence
        0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
        0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, // Padding
    ];

    static ICMPV4_REPLY_PKT: IcmpPacket = IcmpPacket {
        icmp_type: 0,
        request_id: 0x0102,
        seq: 1,
        signature: 0xdeadbeefdeadbeef,
        ts: 0x01020304,
        size: 64 - 20,
    };

    #[test]
    fn test_icmpv4_write() {
        let mut buf: [MaybeUninit<u8>; 4096] = unsafe { MaybeUninit::uninit().assume_init() };
        let n = ICMPV4_REQ_PKT.write(&mut buf);
        let result = unsafe {
            // slice_assume_init_ref
            &*(&buf[..n] as *const [MaybeUninit<u8>] as *const [u8])
        };
        assert_eq!(result, ICMPV4_REQ);
    }

    #[test]
    fn test_arr_to_icmpv4() {
        let pkt = IcmpPacket::try_from(ICMPV4_REPLY).unwrap();
        assert_eq!(pkt, ICMPV4_REPLY_PKT);
    }

    #[test]
    fn test_icmpv4_req_get_sid() {
        let sid = ICMPV4_REQ_PKT.get_sid("127.0.0.1".into());
        assert_eq!(sid, "127.0.0.1-258-1")
    }

    #[test]
    fn test_icmpv4_reply_get_sid() {
        let sid = ICMPV4_REPLY_PKT.get_sid("127.0.0.1".into());
        assert_eq!(sid, "127.0.0.1-258-1")
    }

    #[test]
    fn test_icmpv4_equal_sid() {
        let sid1 = ICMPV4_REQ_PKT.get_sid("127.0.0.1".into());
        let sid2 = ICMPV4_REPLY_PKT.get_sid("127.0.0.1".into());
        assert_eq!(sid1, sid2)
    }
}
