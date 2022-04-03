// ---------------------------------------------------------------------
// Gufo Ping: ICMP packet constructing and parsing
// ---------------------------------------------------------------------
// Copyright (C) 2022, Gufo Labs
// ---------------------------------------------------------------------

use byteorder::{BigEndian, ByteOrder};
use internet_checksum::checksum;
use std::convert::TryFrom;

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
}

// Build IcmpPacket
impl TryFrom<&IcmpPacket> for Vec<u8> {
    type Error = &'static str;

    fn try_from(pkt: &IcmpPacket) -> Result<Self, Self::Error> {
        if pkt.size < 24 {
            return Err("too short");
        }
        let mut buf = Vec::<u8>::with_capacity(pkt.size);
        unsafe {
            buf.set_len(pkt.size);
        }
        // Write type, fill code and checksum with 0
        BigEndian::write_u32(&mut buf, (pkt.icmp_type as u32) << 24);
        // Request id, 2 octets
        BigEndian::write_u16(&mut buf[4..], pkt.request_id);
        // Sequence, 2 octets
        BigEndian::write_u16(&mut buf[6..], pkt.seq);
        // Signature, 8 octets
        BigEndian::write_u64(&mut buf[8..], pkt.signature);
        // Timestamp, 8 octets
        BigEndian::write_u64(&mut buf[16..], pkt.ts);
        // Generate padding, Fill rest by "A"
        if pkt.size > 24 {
            buf[24..].fill(48u8);
        }
        // Calculate checksum
        // RFC-1071
        let cs = checksum(&buf);
        buf[2] = cs[0];
        buf[3] = cs[1];
        Ok(buf)
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
    fn test_icmpv4_to_vec() {
        let out = Vec::<u8>::try_from(&ICMPV4_REQ_PKT).unwrap();
        assert_eq!(out, ICMPV4_REQ);
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
