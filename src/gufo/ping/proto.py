# ---------------------------------------------------------------------
# Gufo Ping: SocketProto
# ---------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# ---------------------------------------------------------------------


# Python modules
from typing import Optional, List, Dict, Protocol


class SocketProto(Protocol):
    """
    SocketWrapper protocol.

    Socket wrapper is the [PyO3](https://pyo3.rs)-wrapped Rust code,
    implementing low-level details of the PingSocket.
    """

    def __init__(self, afi: int) -> None:
        ...

    def set_timeout(self, timeout: int) -> None:
        """
        Set default ping timeout.

        Args:
            timeout: Ping timeout, in nanoseconds.
        """
        ...

    def set_ttl(self, ttl: int) -> None:
        """
        Change outgoing packets' time-to-live field.

        Args:
            ttl: TTL value.
        """
        ...

    def set_tos(self, tos: int) -> None:
        """
        Change outgoing packets' ToS/DSCP field.

        Args:
            tos: ToS value.
        """
        ...

    def set_coarse(self, ct: bool) -> None:
        """
        Switch between the internal timer implemenetation.

        Args:
            ct: Use

                * `CLOCK_MONOTONIC_COARSE` if True
                * `CLOCK_MONOTONIC` if False
        """
        ...

    def set_send_buffer_size(self, size: int) -> None:
        """
        Set outgoing socket's buffer size. If the requested
        size is too big, adjust to proper size.

        Args:
            size: Requested send buffer size, in bytes.
        """
        ...

    def set_recv_buffer_size(self, size: int) -> None:
        """
        Set incoming socket's buffer size. If the requested
        size is too big, adjust to proper size.

        Args:
            size: Requested recv buffer size, in bytes.
        """
        ...

    def set_accelerated(self, a: bool) -> None:
        """
        Enable platform-dependend raw socket processing.

        Args:
            a: Enable or disable an acceleration.
                * True - enable, when platform supports acceleration.
                * False - disable the acceleration.
        """

    def get_fd(self) -> int:  # @todo: Shold be FileDescriptorLike
        """
        Get socket's file descriptor.

        Returns:
            file descriptor for open socket.
        """
        ...

    def send(self, addr: str, request_id: int, seq: int, size: int) -> None:
        """
        Generate and send icmp request packet.

        Args:
            addr: Destination address.
            request_id: ICMP request id.
            seq: ICMP sequental number.
            size: Outgoing packet's size in bytes, including IP header.
        """
        ...

    def recv(self) -> Optional[Dict[str, float]]:
        """
        Receive all awaiting packets.

        Returns:
            * `None` - when no packets received.
            * Dict of `session id` -> `rtt`,
              where `session id` is the string of
              <address>-<request_id>-<seq>,
              and `rtt` - is the measured round-trip-time in nanoseconds.
        """
        ...

    def get_expired(self) -> Optional[List[str]]:
        """
        Get list of sessions with expired timeouts.

        Returns:
            * `None` - when no sessions expired.
            * List of expired sessionn ids, where each session id
              has the format: <address>-<request_id>-<seq>
        """
        ...

    def clean_ip(self, addr: str) -> str:
        """
        Normalize IP address to a stable form.

        Args:
            addr: IP address

        Returns:
            Normalized address
        """
        ...
