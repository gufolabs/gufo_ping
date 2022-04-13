# ---------------------------------------------------------------------
# Gufo Ping: PingSocket implementation
# ---------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# ---------------------------------------------------------------------

# Python modules
from typing import Optional, Dict, cast
from asyncio import Future, get_running_loop, sleep

# Gufo Labs modules
from .proto import SocketProto
from ._fast import SocketWrapper

NS = 1_000_000_000.0


class PingSocket(object):
    """
    Python-side ICMP requests/reply dispatcher for the given address family.
    Wraps Rust socket implementation.

    Args:
        afi: Address Family. Either 4 or 6
        size: Set outgoing packet's size, including IP header.
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
        accelerated: Enable platform-dependend accelerated
            socket processing.
    """

    def __init__(
        self,
        afi: int = 4,
        size: int = 64,
        ttl: Optional[int] = None,
        tos: Optional[int] = None,
        timeout: float = 1.0,
        send_buffer_size: Optional[int] = None,
        recv_buffer_size: Optional[int] = None,
        coarse: bool = False,
        accelerated: bool = True,
    ):
        self.__force_del = False
        if afi != 4 and afi != 6:
            raise ValueError("afi must be 4 or 6")
        # Check settings
        if ttl is not None and (ttl < 1 or ttl > 255):
            raise ValueError("ttl must be in 0..255 range")
        if tos is not None and (tos < 0 or tos > 255):
            raise ValueError("tos must be in 0..255 range")
        #
        self.__size = size
        # Create and initialize wrapped socket
        self.__sock: SocketProto = cast(SocketProto, SocketWrapper(afi))
        self.__sock.set_timeout(int(timeout * NS))
        if ttl is not None:
            self.__sock.set_ttl(ttl)
        if tos is not None:
            self.__sock.set_tos(tos)
        if send_buffer_size is not None:
            self.__sock.set_send_buffer_size(send_buffer_size)
        if recv_buffer_size is not None:
            self.__sock.set_recv_buffer_size(recv_buffer_size)
        if coarse:
            self.__sock.set_coarse(True)
        if accelerated:
            self.__sock.set_accelerated(True)
        self.__timeout = timeout
        self.__sock_fd = self.__sock.get_fd()
        #  <addr>-<request id>-<seq> -> future
        self.__sessions: Dict[str, Future[Optional[float]]] = {}
        # Install response reader
        self.__force_del = True
        get_running_loop().add_reader(self.__sock_fd, self.__on_read)
        # Install deadline cleaner
        self.__cleanup_task = get_running_loop().create_task(self.__cleanup())

    def __del__(self) -> None:
        """
        Perform cleanup on delete:

        * Cancel expiration task.
        * Remove socket reader.
        """
        if not self.__force_del:
            return
        try:
            # Unsubscribe reader
            # get_running_loop() may raise Runtime Error
            get_running_loop().remove_reader(self.__sock_fd)
            # Stop cleanup task
            if self.__cleanup_task is not None:
                self.__cleanup_task.cancel()
        except RuntimeError:  # pragma: no cover
            pass  # Loop is already closed

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
        size: Optional[int] = None,
        request_id: int = 0,
        seq: int = 0,
    ) -> Optional[float]:
        """
        Send ICMP echo request and await for result.

        Args:
            addr: Socket to ping.
            size: Packet size in bytes, including IP header.
            request_id: ICMP request id.
            seq: ICMP sequental number.
        """
        if ":" in addr:
            # Convert IPv6 address to compact form
            addr = self.__sock.clean_ip(addr)
        sid = f"{addr}-{request_id}-{seq}"
        fut: Future[Optional[float]] = get_running_loop().create_future()
        # Build and send the packet
        self.__sock.send(addr, request_id, seq, size or self.__size)
        # Install future in the sessions
        self.__sessions[sid] = fut
        # Await response or timeout
        return await fut

    def __on_read(self) -> None:
        """
        Handle socket read event.
        """
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
                fut.set_result(float(rtt) / NS)

    async def __cleanup(self) -> None:
        """
        Check for expired sessions and close them.
        """
        while True:
            # Wait for next cycle
            await sleep(self.__timeout)
            # Get a list of exired sids
            expired = self.__sock.get_expired()
            if not expired:
                continue
            # Iterate over expired sids
            for sid in expired:
                # Find and pop the future by single call
                fut = self.__sessions.pop(sid, None)
                if fut:
                    # Pass None to indicate the timeout
                    fut.set_result(None)
