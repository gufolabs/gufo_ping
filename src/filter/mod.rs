// ---------------------------------------------------------------------
// Gufo Ping: Socket filter utilities
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use socket2::Socket;
use std::io;

#[cfg(target_os = "linux")]
pub(crate) mod filter_linux;

#[derive(Clone, Copy)]
pub(crate) enum Filter {
    #[allow(unused)]
    None, // No filter
    #[cfg(target_os = "linux")]
    LinuxRaw4, // Linux RAW socket, IPv4
    #[cfg(target_os = "linux")]
    LinuxRaw6, // Linux RAW socket, IPv6
}

impl Filter {
    #[inline(always)]
    pub(crate) fn attach_filter(self, sock: &Socket, signature: u64) -> io::Result<()> {
        match self {
            Filter::None => Ok(()),
            #[cfg(target_os = "linux")]
            Filter::LinuxRaw4 => filter_linux::attach_raw4(sock, signature),
            #[cfg(target_os = "linux")]
            Filter::LinuxRaw6 => filter_linux::attach_raw6(sock, signature),
        }
    }
}
