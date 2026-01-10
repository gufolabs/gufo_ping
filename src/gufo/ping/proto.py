# ---------------------------------------------------------------------
# Gufo Ping: SocketProto
# ---------------------------------------------------------------------
# Copyright (C) 2022-26, Gufo Labs
# ---------------------------------------------------------------------

"""SocketWrapper protocol definition."""

# Python modules
from typing import Dict, Optional, Protocol


class SocketProto(Protocol):
    """
    SocketWrapper protocol.

    Socket wrapper is the [PyO3](https://pyo3.rs)-wrapped Rust code,
    implementing low-level details of the PingSocket.
    """

    def __init__(self, policy: int, timeout_ns: int, coarse: bool) -> None: ...

    def bind(self, addr: str) -> None:
        """
        Bind to source address.

        Args:
            addr: Source Address
        """

    def set_ttl(self, ttl: int) -> None:
        """
        Change outgoing packets' time-to-live field.

        Args:
            ttl: TTL value.
        """

    def set_unicast_hops(self, ttl: int) -> None:
        """
        Change outgoing packets' unicast hops (IPv6 TTL).

        Args:
            ttl: TTL value.
        """

    def set_tos(self, tos: int) -> None:
        """
        Change outgoing packets' ToS/DSCP field.

        Args:
            tos: ToS value.
        """

    def set_tclass(self, tclass: int) -> None:
        """
        Change outgoing packets' IPv6 TCLASS field.

        Args:
            tclass: TCLASS value.
        """

    def set_send_buffer_size(self, size: int) -> None:
        """
        Set outgoing socket's buffer size.

        If the requested
        size is too big, adjust to proper size.

        Args:
            size: Requested send buffer size, in bytes.
        """

    def set_recv_buffer_size(self, size: int) -> None:
        """
        Set incoming socket's buffer size.

        If the requested
        size is too big, adjust to proper size.

        Args:
            size: Requested recv buffer size, in bytes.
        """

    def get_fd(
        self,
    ) -> int:  # @todo: Shold be FileDescriptorLike
        """
        Get socket's file descriptor.

        Returns:
            file descriptor for open socket.
        """

    def send(self, addr: str, size: int) -> int:
        """
        Generate and send icmp request packet.

        Args:
            addr: Destination address.
            size: Outgoing packet's size in bytes, including IP header.

        Returns:
            session id.
        """

    def recv(self) -> Optional[Dict[int, float]]:
        """
        Receive all awaiting packets.

        Returns:
            * `None` - when no packets received.
            * Dict of `session id` -> `rtt`,
                where `session id` is the value, returned by `send`
                and `rtt` - is the measured round-trip-time in nanoseconds.
        """

    def clean_ip(self, addr: str) -> str:
        """
        Normalize IP address to a stable form.

        Args:
            addr: IP address

        Returns:
            Normalized address
        """
