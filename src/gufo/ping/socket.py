# ---------------------------------------------------------------------
# Gufo Ping: PingSocket implementation
# ---------------------------------------------------------------------
# Copyright (C) 2022-26, Gufo Labs
# ---------------------------------------------------------------------

"""
PingSocket implementation.

Attributes:
    IPv4: IPv4 address family.
    IPv6: IPv6 address family.
"""

# Python modules
from asyncio import Future, TimerHandle, get_running_loop
from contextlib import suppress
from enum import IntEnum
from typing import cast

# Gufo Labs modules
from ._fast import (
    PS_DGRAM,
    PS_DGRAM_RAW,
    PS_IPV4,
    PS_IPV6,
    PS_RAW,
    PS_RAW_DGRAM,
    SocketWrapper,
)
from .proto import SocketProto

NS = 1_000_000_000.0
IPv4 = 4
IPv6 = 6
MAX_TTL = 255
MAX_TOS = 255

POLICY_SHIFT = {IPv4: PS_IPV4, IPv6: PS_IPV6}


class SelectionPolicy(IntEnum):
    """
    Probe selection policy.

    Allows automatical selection between implementations:

    * `RAW` - Raw sockets, requires root user (POSIX platforms)
        or CAP_NET_RAW capability (Linux).
    * `DGRAM` - ICMP datagram socket (Linux only). Requires
        `net.ipv4.ping_group_range` to contain process' group.

    Attributes:
        RAW: Use RAW only.
        RAW_DGRAM: Try RAW, fallback to DGRAM.
        DGRAM_RAW: Try DGRAM, fallback to RAW.
        DGRAM: Use DGRAM only.
    """

    RAW = PS_RAW
    RAW_DGRAM = PS_RAW_DGRAM
    DGRAM_RAW = PS_DGRAM_RAW
    DGRAM = PS_DGRAM


class PingSocket(object):
    """
    Python API to Gufo Ping internals.

    Python-side ICMP requests/reply dispatcher for the given address family.
    Wraps Rust socket implementation.

    Args:
        afi: Address Family. Either IPv4 or IPv6.
        policy: Probe selection policy.
        size: Set outgoing packet's size, including IP header.
        src_addr: Optional source address of outgoing packets.
        ttl: Set outgoing packet's TTL.
            Use OS defaults when empty.
        tos: Set DSCP/TOS field to outgoing packets.
            Use OS defaults when empty.
        timeout: Default timeout in seconds.
        send_buffer_size: Send buffer size.
            Use OS defaults when empty.
        recv_buffer_size: Receive buffer size.
            Use OS defaults when empty.
        coarse: Use CLOCK_MONOTONIC_COARSE when set,
            fall back to CLOCK_MONOTONIC otherwise.
    """

    VALID_AFI = (IPv4, IPv6)

    def __init__(
        self,
        afi: int = IPv4,
        policy: SelectionPolicy = SelectionPolicy.RAW,
        size: int = 64,
        src_addr: str | None = None,
        ttl: int | None = None,
        tos: int | None = None,
        timeout: float = 1.0,
        send_buffer_size: int | None = None,
        recv_buffer_size: int | None = None,
        coarse: bool = False,
    ) -> None:
        if afi not in self.VALID_AFI:
            msg = f"afi must be {IPv4} or {IPv6}"
            raise ValueError(msg)
        # Check settings
        if ttl is not None and (ttl < 1 or ttl > MAX_TTL):
            msg = f"ttl must be in 0..{MAX_TTL} range"
            raise ValueError(msg)
        if tos is not None and (tos < 0 or tos > MAX_TOS):
            msg = f"tos must be in 0..{MAX_TOS} range"
            raise ValueError(msg)
        self.__size = size
        # Calculate effective policy
        selection_policy = POLICY_SHIFT[afi] + policy.value
        # Create and initialize wrapped socket
        self.__sock: SocketProto = cast(
            SocketProto,
            SocketWrapper(selection_policy, int(timeout * NS), coarse),
        )
        if src_addr:
            self.__sock.bind(src_addr)
        if ttl is not None:
            if afi == IPv4:
                self.__sock.set_ttl(ttl)
            elif afi == IPv6:
                self.__sock.set_unicast_hops(ttl)
        if tos is not None:
            if afi == IPv4:
                self.__sock.set_tos(tos)
            elif afi == IPv6:
                self.__sock.set_tclass(tos)
        if send_buffer_size is not None:
            self.__sock.set_send_buffer_size(send_buffer_size)
        if recv_buffer_size is not None:
            self.__sock.set_recv_buffer_size(recv_buffer_size)
        self.__sock_fd = self.__sock.get_fd()
        #  session id -> future
        self.__sessions: dict[int, Future[float | None]] = {}
        # Install response reader
        get_running_loop().add_reader(self.__sock_fd, self._on_read)
        self._timeout_handler: TimerHandle | None = None
        self.__timeout = timeout

    def __del__(self) -> None:
        """Perform cleanup on delete."""
        fd = getattr(self, "__sock_fd", None)
        if fd:
            with suppress(RuntimeError):
                get_running_loop().remove_reader(self.__sock_fd)

    def clean_ip(self, addr: str) -> str:
        """
        Normalize IP address to a stable form.

        Args:
            addr: IP address

        Returns:
            Normalized address
        """
        return self.__sock.clean_ip(addr)

    async def ping(
        self,
        addr: str,
        size: int | None = None,
    ) -> float | None:
        """
        Send ICMP echo request and await for result.

        Args:
            addr: Socket to ping.
            size: Packet size in bytes, including IP header.
        """
        if ":" in addr:
            # Convert IPv6 address to compact form
            addr = self.__sock.clean_ip(addr)
        # Build and send the packet
        try:
            sid = self.__sock.send(addr, size or self.__size)
        except OSError:
            # Some kernels raise OSError (Network Unreachable)
            # when cannot find the route. Treat them as losses.
            return None
        # Install future in the sessions
        fut: Future[float | None] = get_running_loop().create_future()
        self.__sessions[sid] = fut
        # Trigger timeout handler when necessary
        if not self._timeout_handler:
            self._start_timeout_checker()
        # Await response or timeout. set by _on_read
        return await fut

    def _on_read(self) -> None:
        """Handle socket read event."""
        # Get bulk read info from Rust side
        seen = self.__sock.recv()
        if seen is None:
            return
        # seen is the dict of sid -> rtt
        for sid, rtt in seen.items():
            # Find and pop the future in single call
            fut = self.__sessions.pop(sid, None)
            if fut:
                # Pass rtt to the future, unblock await in `ping`
                fut.set_result(float(rtt) / NS if rtt else None)

    def _check_timeouts(self) -> None:
        """Check for pending timeouts."""
        self._timeout_handler = None
        self._on_read()
        if self.__sessions:
            self._start_timeout_checker()

    def _start_timeout_checker(self) -> None:
        """Install timeout checker."""
        self._timeout_handler = get_running_loop().call_later(
            self.__timeout, self._check_timeouts
        )
