// ---------------------------------------------------------------------
// Gufo Ping: ICMP protocol variants and packet handling
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------
use crate::{PingError, PingResult, filter::Filter, slice};
use byteorder::{BigEndian, ByteOrder};
use internet_checksum::checksum;
use rand::Rng;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    io,
    mem::MaybeUninit,
    net::{SocketAddrV4, SocketAddrV6},
    sync::OnceLock,
};

// Sizes
const IPV4_HEADER_SIZE: usize = 20; // IPv4 header size
const IPV6_HEADER_SIZE: usize = 40; // IPv6 header size
const ICMP_HEADER_SIZE: usize = 8; // ICMP heqder size
const ICMP_PAYLOAD_SIZE: usize = 16; // Session ID (8) + Timestamp (8)
const PADDING: u8 = 48; // Payload padding
const PADDING_OFFSET: usize = ICMP_HEADER_SIZE + ICMP_PAYLOAD_SIZE;

/// ```text
///  ICMP protocol layout
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
const ICMP_TYPE_OFFSET: usize = 0;
const CHECKSUM_OFFSET: usize = 2;
// const REQUEST_ID_OFFSET: usize = 4;
const SEQUENCE_OFFSET: usize = 6;
const SIGNATURE_OFFSET: usize = ICMP_HEADER_SIZE;
const TIMESTAMP_OFFSET: usize = SIGNATURE_OFFSET + 8;

// Protocol configuration
pub(crate) struct Proto {
    // Platform configuration.
    // If false, proto is not supported on this platform.
    // If true -- additionaly check availability.
    has_platform_support: bool,
    // Socket configuration
    domain: Domain,
    ty: Type,
    protocol: Protocol,
    filter: Filter,
    // Underlying IP protocol configuration
    ip_header_size: usize,
    // ICMP protocol configuration
    icmp_request_type: u8,
    icmp_reply_type: u8,
    // ICMP handling configuration
    // Number of octets to skip when parsing response.
    // Set to IP header size if recv returns IP header.
    skip_reply: usize,
    // Calculate checksum in user space
    require_checksum: bool,
}

// Protocol configurations
#[repr(usize)]
#[derive(Clone)]
enum ProtocolItem {
    IPv4Raw = 0,
    IPv4Dgram = 1,
    IPv6Raw = 2,
    IPv6Dgram = 3,
}

const N_PROTOCOLS: usize = 4;
static PROTOCOLS: [Proto; N_PROTOCOLS] = [
    // IPv4, RAW socket
    Proto {
        has_platform_support: true, // All POSIX platforms
        domain: Domain::IPV4,
        ty: Type::RAW,
        protocol: Protocol::ICMPV4,
        #[cfg(target_os = "linux")]
        filter: Filter::LinuxRaw4,
        #[cfg(not(target_os = "linux"))]
        filter: Filter::None,
        ip_header_size: IPV4_HEADER_SIZE,
        icmp_request_type: 8,
        icmp_reply_type: 0,
        skip_reply: 20, // recv returns header.
        require_checksum: true,
    },
    // IPv4, DGRAM
    Proto {
        #[cfg(target_os = "linux")]
        has_platform_support: true,
        #[cfg(not(target_os = "linux"))]
        has_platform_support: false,
        domain: Domain::IPV4,
        ty: Type::DGRAM,
        protocol: Protocol::ICMPV4,
        filter: Filter::None,
        ip_header_size: IPV4_HEADER_SIZE,
        icmp_request_type: 8,
        icmp_reply_type: 0,
        skip_reply: 0,
        require_checksum: false,
    },
    // IPv6, RAW Socket
    Proto {
        has_platform_support: true, // All POSIX platforms
        domain: Domain::IPV6,
        ty: Type::RAW,
        protocol: Protocol::ICMPV6,
        #[cfg(target_os = "linux")]
        filter: Filter::LinuxRaw6,
        #[cfg(not(target_os = "linux"))]
        filter: Filter::None,
        ip_header_size: IPV6_HEADER_SIZE,
        icmp_request_type: 128,
        icmp_reply_type: 129,
        skip_reply: 0, // recv doesn't return header.
        require_checksum: true,
    },
    // IPv6 DGRAM
    Proto {
        #[cfg(target_os = "linux")]
        has_platform_support: true,
        #[cfg(not(target_os = "linux"))]
        has_platform_support: false,
        domain: Domain::IPV6,
        ty: Type::DGRAM,
        protocol: Protocol::ICMPV6,
        filter: Filter::None,
        ip_header_size: IPV6_HEADER_SIZE,
        icmp_request_type: 128,
        icmp_reply_type: 129,
        skip_reply: 0, // recv doesn't return header.
        require_checksum: false,
    },
];

static IS_AVAILABLE: [OnceLock<bool>; N_PROTOCOLS] = [
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
];

impl Proto {
    // Create proper socket
    // Returns Socket, signature
    #[inline(always)]
    pub(crate) fn create_socket(&self) -> io::Result<(Socket, u64)> {
        let io = Socket::new(self.domain, self.ty, Some(self.protocol))?;
        let signature = rand::rng().random();
        self.filter.attach_filter(&io, signature)?;
        io.set_nonblocking(true)?;
        Ok((io, signature))
    }

    // Check if protocol is available
    pub(crate) fn is_available(&self) -> bool {
        self.has_platform_support && self.create_socket().is_ok()
    }

    // Serialize ICMP request to buffer
    #[inline(always)]
    pub(crate) fn encode_request<'a>(
        &self,
        probe: Probe,
        buf: &'a mut [MaybeUninit<u8>],
        size: usize,
    ) -> &'a [u8] {
        let size = size - self.ip_header_size; // Adjust to packet header
        let buf = slice::slice_assume_init_mut(&mut buf[..size]);
        // Write:
        // * type
        // * Zeroes in code and checksum place
        // * Request id
        // * Sequence
        BigEndian::write_u64(
            buf,
            ((self.icmp_request_type as u64) << 56)
                | ((probe.get_request_id() as u64) << 16)
                | (probe.seq as u64),
        );
        // Signature, 8 octets
        BigEndian::write_u64(&mut buf[SIGNATURE_OFFSET..], probe.signature);
        // Timestamp, 8 octets
        BigEndian::write_u64(&mut buf[TIMESTAMP_OFFSET..], probe.ts);
        // Generate padding, Fill rest by "A"
        if size > PADDING_OFFSET {
            buf[PADDING_OFFSET..].fill(PADDING);
        }
        // Calculate checksum
        // RFC-1071
        if self.require_checksum {
            let cs = checksum(buf);
            buf[CHECKSUM_OFFSET] = cs[0];
            buf[CHECKSUM_OFFSET + 1] = cs[1];
        }
        buf
    }
    // deserialize reply
    pub(crate) fn decode_reply(&self, buf: &[u8]) -> Option<Probe> {
        let buf = &buf[self.skip_reply..];
        if buf.len() < PADDING_OFFSET {
            return None;
        }
        if buf[ICMP_TYPE_OFFSET] != self.icmp_reply_type {
            return None;
        }
        // @todo: request id must match two lower bits of signature
        Some(Probe {
            seq: BigEndian::read_u16(&buf[SEQUENCE_OFFSET..]),
            signature: BigEndian::read_u64(&buf[SIGNATURE_OFFSET..]),
            ts: BigEndian::read_u64(&buf[TIMESTAMP_OFFSET..]),
        })
    }
    #[inline(always)]
    pub(crate) fn to_ip(&self, addr: &str) -> PingResult<String> {
        match self.domain {
            Domain::IPV4 => Ok(SocketAddrV4::new(addr.parse()?, 0).ip().to_string()),
            Domain::IPV6 => Ok(SocketAddrV6::new(addr.parse()?, 0, 0, 0).ip().to_string()),
            _ => Err(PingError::InvalidAddr),
        }
    }
    #[inline(always)]
    pub(crate) fn to_sockaddr(&self, addr: &str) -> PingResult<SockAddr> {
        match self.domain {
            Domain::IPV4 => Ok(SocketAddrV4::new(addr.parse()?, 0).into()),
            Domain::IPV6 => Ok(SocketAddrV6::new(addr.parse()?, 0, 0, 0).into()),
            _ => Err(PingError::InvalidAddr),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Probe {
    seq: u16,
    signature: u64,
    ts: u64,
}

impl Probe {
    pub fn new(seq: u16, signature: u64, ts: u64) -> Self {
        Probe { seq, signature, ts }
    }

    #[inline]
    pub fn get_request_id(&self) -> u16 {
        self.signature as u16
    }

    #[inline]
    pub fn get_seq(&self) -> u16 {
        self.seq
    }

    pub fn get_signature(&self) -> u64 {
        self.signature
    }

    pub fn get_ts(&self) -> u64 {
        self.ts
    }
}

pub(crate) const PS_RAW: u8 = 0;
pub(crate) const PS_RAW_DGRAM: u8 = 1;
pub(crate) const PS_DGRAM_RAW: u8 = 2;
pub(crate) const PS_DGRAM: u8 = 3;
pub(crate) const PS_IPV4: u8 = 0;
pub(crate) const PS_IPV6: u8 = 4;

// Protocol selection policy
#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub(crate) enum SelectionPolicy {
    IPv4Raw = PS_IPV4 + PS_RAW,
    IPv4RawDgram = PS_IPV4 + PS_RAW_DGRAM,
    IPv4DgramRaw = PS_IPV4 + PS_DGRAM_RAW,
    IPv4Dgram = PS_IPV4 + PS_DGRAM,
    IPv6Raw = PS_IPV6 + PS_RAW,
    IPv6RawDgram = PS_IPV6 + PS_RAW_DGRAM,
    IPv6DgramRaw = PS_IPV6 + PS_DGRAM_RAW,
    IPv6Dgram = PS_IPV6 + PS_DGRAM,
}

impl TryFrom<u8> for SelectionPolicy {
    type Error = PingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SelectionPolicy::IPv4Raw),
            1 => Ok(SelectionPolicy::IPv4RawDgram),
            2 => Ok(SelectionPolicy::IPv4DgramRaw),
            3 => Ok(SelectionPolicy::IPv4Dgram),
            4 => Ok(SelectionPolicy::IPv6Raw),
            5 => Ok(SelectionPolicy::IPv6RawDgram),
            6 => Ok(SelectionPolicy::IPv6DgramRaw),
            7 => Ok(SelectionPolicy::IPv6Dgram),
            _ => Err(PingError::InvalidPolicy),
        }
    }
}

impl SelectionPolicy {
    //
    fn candidates(self) -> &'static [usize] {
        match self {
            SelectionPolicy::IPv4Raw => &[ProtocolItem::IPv4Raw as usize],
            SelectionPolicy::IPv4RawDgram => &[
                ProtocolItem::IPv4Raw as usize,
                ProtocolItem::IPv4Dgram as usize,
            ],
            SelectionPolicy::IPv4DgramRaw => &[
                ProtocolItem::IPv4Dgram as usize,
                ProtocolItem::IPv4Raw as usize,
            ],
            SelectionPolicy::IPv4Dgram => &[ProtocolItem::IPv4Dgram as usize],
            SelectionPolicy::IPv6Raw => &[ProtocolItem::IPv6Raw as usize],
            SelectionPolicy::IPv6RawDgram => &[
                ProtocolItem::IPv6Raw as usize,
                ProtocolItem::IPv6Dgram as usize,
            ],
            SelectionPolicy::IPv6DgramRaw => &[
                ProtocolItem::IPv6Dgram as usize,
                ProtocolItem::IPv6Raw as usize,
            ],
            SelectionPolicy::IPv6Dgram => &[ProtocolItem::IPv6Dgram as usize],
        }
    }

    // Select first appropriate protocol from items.
    fn select_protocol(candidates: &[usize]) -> PingResult<&'static Proto> {
        if candidates.is_empty()
            || candidates.iter().all(|&idx| {
                PROTOCOLS
                    .get(idx)
                    .is_none_or(|proto| !proto.has_platform_support)
            })
        {
            return Err(PingError::NotImplemented);
        }
        for &candidate in candidates.iter() {
            if *IS_AVAILABLE[candidate].get_or_init(|| PROTOCOLS[candidate].is_available()) {
                return Ok(&PROTOCOLS[candidate]);
            }
        }
        Err(PingError::PermissionDenied)
    }
}

impl TryFrom<SelectionPolicy> for &'static Proto {
    type Error = PingError;

    fn try_from(value: SelectionPolicy) -> Result<Self, Self::Error> {
        let candidates = value.candidates();
        SelectionPolicy::select_protocol(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slice::get_buffer_mut;

    const TEST_REQUEST_ID: u16 = 0xbeef;
    const TEST_SEQ: u16 = 1;
    const TEST_SIGNATURE: u64 = 0xdeadbeef;
    const TEST_TIMESTAMP: u64 = 0x01020304;

    #[test]
    fn test_settings() {
        assert_eq!(PROTOCOLS.len(), IS_AVAILABLE.len());
    }

    #[test]
    fn test_selection_policy_try_from() {
        let expected = [
            SelectionPolicy::IPv4Raw,
            SelectionPolicy::IPv4RawDgram,
            SelectionPolicy::IPv4DgramRaw,
            SelectionPolicy::IPv4Dgram,
            SelectionPolicy::IPv6Raw,
            SelectionPolicy::IPv6RawDgram,
            SelectionPolicy::IPv6DgramRaw,
            SelectionPolicy::IPv6Dgram,
        ];
        for (policy_id, expected) in expected.iter().enumerate() {
            let policy = SelectionPolicy::try_from(policy_id as u8).unwrap();
            assert_eq!(policy as u8, *expected as u8);
        }
    }

    #[test]
    fn test_selection_policy_try_from_error() {
        assert!(SelectionPolicy::try_from(0xffu8).is_err())
    }

    #[test]
    fn test_selection_policy_candidates() {
        let policies = [
            SelectionPolicy::IPv4Raw,
            SelectionPolicy::IPv4RawDgram,
            SelectionPolicy::IPv4DgramRaw,
            SelectionPolicy::IPv4Dgram,
            SelectionPolicy::IPv6Raw,
            SelectionPolicy::IPv6RawDgram,
            SelectionPolicy::IPv6DgramRaw,
            SelectionPolicy::IPv6Dgram,
        ];
        let expected = [
            vec![ProtocolItem::IPv4Raw],
            vec![ProtocolItem::IPv4Raw, ProtocolItem::IPv4Dgram],
            vec![ProtocolItem::IPv4Dgram, ProtocolItem::IPv4Raw],
            vec![ProtocolItem::IPv4Dgram],
            vec![ProtocolItem::IPv6Raw],
            vec![ProtocolItem::IPv6Raw, ProtocolItem::IPv6Dgram],
            vec![ProtocolItem::IPv6Dgram, ProtocolItem::IPv6Raw],
            vec![ProtocolItem::IPv6Dgram],
        ];
        for (p, e) in policies.iter().zip(expected.iter()) {
            let expected_indexes: Vec<usize> = e.iter().map(|x| x.clone() as usize).collect();
            assert_eq!(p.candidates(), expected_indexes);
        }
    }

    #[test]
    fn test_v4_raw_encode1() {
        const SIZE: usize = 44;
        let proto = &PROTOCOLS[ProtocolItem::IPv4Raw as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                8, 0, 0x97, 0x6B, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ]
        )
    }

    #[test]
    fn test_v4_raw_encode2() {
        const SIZE: usize = 64;
        let proto = &PROTOCOLS[ProtocolItem::IPv4Raw as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                8, 0, 0xB5, 0x89, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ]
        )
    }

    #[test]
    fn test_v4_raw_decode1() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Raw as usize];
        let probe = proto
            .decode_reply(&[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, // IP header, faked
                0, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v4_raw_decode2() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Raw as usize];
        let probe = proto
            .decode_reply(&[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, // IP header, faked
                0, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v4_raw_decode_too_short() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Raw as usize];
        let probe = proto.decode_reply(&[
            0, // IP header, faked
            0, 0, 0, 0, // Type, Code, Checksum (faked)
            0xBE, 0xEF, 0, 1, // Request id, sequence
            0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
            0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
        ]);
        assert!(probe.is_none());
    }

    #[test]
    fn test_v4_raw_decode_invalid_type() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Raw as usize];
        let probe = proto.decode_reply(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // IP header, faked
            8, 0, 0, 0, // Type, Code, Checksum (faked)
            0xBE, 0xEF, 0, 1, // Request id, sequence
            0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
            0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
        ]);
        assert!(probe.is_none())
    }
    #[test]
    fn test_v4_dgram_encode1() {
        const SIZE: usize = 44;
        let proto = &PROTOCOLS[ProtocolItem::IPv4Dgram as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                8, 0, 0, 0, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ]
        )
    }

    #[test]
    fn test_v4_dgram_encode2() {
        const SIZE: usize = 64;
        let proto = &PROTOCOLS[ProtocolItem::IPv4Dgram as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                8, 0, 0, 0, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ]
        )
    }

    #[test]
    fn test_v4_dgram_decode1() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Dgram as usize];
        let probe = proto
            .decode_reply(&[
                0, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v4_dgram_decode2() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Dgram as usize];
        let probe = proto
            .decode_reply(&[
                0, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v4_dgram_decode_too_short() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Dgram as usize];
        let probe = proto.decode_reply(&[
            0, // IP header, faked
            0, 0, 0, 0, // Type, Code, Checksum (faked)
            0xBE, 0xEF, 0, 1, // Request id, sequence
        ]);
        assert!(probe.is_none());
    }

    #[test]
    fn test_v4_dgram_decode_invalid_type() {
        let proto = &PROTOCOLS[ProtocolItem::IPv4Dgram as usize];
        let probe = proto.decode_reply(&[
            8, 0, 0, 0, // Type, Code, Checksum (faked)
            0xBE, 0xEF, 0, 1, // Request id, sequence
            0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
            0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
        ]);
        assert!(probe.is_none())
    }
    #[test]
    fn test_v6_raw_encode1() {
        const SIZE: usize = 64;
        let proto = &PROTOCOLS[ProtocolItem::IPv6Raw as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                0x80, 0, 0x1F, 0x6B, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ]
        )
    }

    #[test]
    fn test_v6_raw_encode2() {
        const SIZE: usize = 84;
        let proto = &PROTOCOLS[ProtocolItem::IPv6Raw as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                0x80, 0, 0x3D, 0x89, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ]
        )
    }

    #[test]
    fn test_v6_raw_decode1() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Raw as usize];
        let probe = proto
            .decode_reply(&[
                0x81, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v6_raw_decode2() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Raw as usize];
        let probe = proto
            .decode_reply(&[
                0x81, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v6_raw_decode_too_short() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Raw as usize];
        let probe = proto.decode_reply(&[
            0, // Short packet
        ]);
        assert!(probe.is_none());
    }

    #[test]
    fn test_v6_raw_decode_invalid_type() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Raw as usize];
        let probe = proto.decode_reply(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // IP header, faked
            8, 0, 0, 0, // Type, Code, Checksum (faked)
            0xBE, 0xEF, 0, 1, // Request id, sequence
            0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
            0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
        ]);
        assert!(probe.is_none())
    }
    #[test]
    fn test_v6_dgram_encode1() {
        const SIZE: usize = 64;
        let proto = &PROTOCOLS[ProtocolItem::IPv6Dgram as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                0x80, 0, 0, 0, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ]
        )
    }

    #[test]
    fn test_v6_dgram_encode2() {
        const SIZE: usize = 84;
        let proto = &PROTOCOLS[ProtocolItem::IPv6Dgram as usize];
        let mut buf = get_buffer_mut();
        let buf = proto.encode_request(
            Probe::new(TEST_SEQ, TEST_SIGNATURE, TEST_TIMESTAMP),
            &mut buf,
            SIZE,
        );
        assert_eq!(
            buf,
            &[
                0x80, 0, 0, 0, // Type, Code, Checksum
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ]
        )
    }

    #[test]
    fn test_v6_dgram_decode1() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Dgram as usize];
        let probe = proto
            .decode_reply(&[
                0x81, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v6_dgram_decode2() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Dgram as usize];
        let probe = proto
            .decode_reply(&[
                0x81, 0, 0, 0, // Type, Code, Checksum (faked)
                0xBE, 0xEF, 0, 1, // Request id, sequence
                0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
                0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
                0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
            ])
            .unwrap();
        assert_eq!(probe.get_request_id(), TEST_REQUEST_ID);
        assert_eq!(probe.get_seq(), TEST_SEQ);
        assert_eq!(probe.get_signature(), TEST_SIGNATURE);
        assert_eq!(probe.get_ts(), TEST_TIMESTAMP);
    }
    #[test]
    fn test_v6_dgram_decode_too_short() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Dgram as usize];
        let probe = proto.decode_reply(&[
            0, // Short packet
        ]);
        assert!(probe.is_none());
    }

    #[test]
    fn test_v6_dgram_decode_invalid_type() {
        let proto = &PROTOCOLS[ProtocolItem::IPv6Dgram as usize];
        let probe = proto.decode_reply(&[
            8, 0, 0, 0, // Type, Code, Checksum (faked)
            0xBE, 0xEF, 0, 1, // Request id, sequence
            0, 0, 0, 0, 0xDE, 0xAD, 0xBE, 0xEF, // Signature
            0, 0, 0, 0, 1, 2, 3, 4, // Timestamp
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, // Padding, 20x"A"
        ]);
        assert!(probe.is_none())
    }
}
