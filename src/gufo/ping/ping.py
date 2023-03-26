# ---------------------------------------------------------------------
# Gufo Ping: Ping implementation
# ---------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# ---------------------------------------------------------------------

"""Ping client implementation."""

# Python modules
import asyncio
import itertools
import random
from time import perf_counter
from typing import AsyncIterable, Dict, Iterable, Optional, Tuple, Union

# Gufo Labs modules
from .socket import IPv4, IPv6, PingSocket


class Ping(object):
    """
    High-performance asyncronous ICMPv4/ICMPv6 ping client.

    Args:
        size: Set outgoing packet's size, including IP header.
        src_addr: Set source address for outgoing packets.
            Depends upon address family. May be one of:
            * None - detect source address automatically.
            * str - containing source address for one address family.
            * Iterable of strings, containing multiple addresses
                which to be distributed among the address families.
                First address for given address family will be used.
        ttl: Set outgoing packet's TTL.
            Use OS defaults when empty.

            !!! warning

                This option is ignored on IPv6 socket due to
                issue [#4](https://github.com/gufolabs/gufo_ping/issues/4)
        tos: Set DSCP/TOS/TCLASS field to outgoing packets.
            Use OS defaults when empty.

            !!! warning

                This option is ignored on IPv6 socket due to
                issue [#2](https://github.com/gufolabs/gufo_ping/issues/2)
        timeout: Default timeout in seconds.
        send_buffer_size: Send buffer size.
            Use OS defaults when empty.
        recv_buffer_size: Receive buffer size.
            Use OS defaults when empty.
        coarse: Use CLOCK_MONOTONIC_COARSE when set,
            fall back to CLOCK_MONOTONIC otherwise.
        accelerated: Enable platform-dependend accelerated
            socket processing.

    Note:
        Opening the Raw Socket may require super-user priveleges
        or additional permissions. Refer to the operation system's
        documentation for details.

    Example:
        Ping single packet.

        ``` py
        from gufo.ping import Ping

        async def ping(address):
            p = Ping()
            rtt = await p.ping(address)
            print(rtt)
        ```

        Ping multiple packets.

        ``` py
        from gufo.ping import Ping

        async def ping(address):
            p = Ping()
            async for rtt in p.iter_rtt(address):
                print(rtt)
        ```
    """

    request_id = itertools.count(random.randint(0, 0xFFFF))

    def __init__(
        self: "Ping",
        size: int = 64,
        src_addr: Union[None, str, Iterable[str]] = None,
        ttl: Optional[int] = None,
        tos: Optional[int] = None,
        timeout: float = 1.0,
        send_buffer_size: Optional[int] = None,
        recv_buffer_size: Optional[int] = None,
        coarse: bool = False,
        accelerated: bool = True,
    ) -> None:
        self.__size = size
        self.__src_addr = self._get_src_addr(src_addr)
        self.__ttl = ttl
        self.__tos = tos
        self.__timeout = timeout
        self.__send_buffer_size = send_buffer_size
        self.__recv_buffer_size = recv_buffer_size
        self.__coarse = coarse
        self.__accelerated = accelerated
        self.__sockets: Dict[int, PingSocket] = {}

    @staticmethod
    def _get_afi(address: str) -> int:
        """
        Get address family (AFI) for a given address.

        Args:
            address: Address to ping.

        Returns:
            * `4` for IPv4
            * `6` for IPv6
        """
        if ":" in address:
            return IPv6
        return IPv4

    @staticmethod
    def _get_src_addr(addr: Union[None, str, Iterable[str]]) -> Dict[int, str]:
        """
        Parse source addresses.

        Parse source addresses and distribute them around address families.

        Args:
            addr: One of:
                * None - detect source address automatically.
                * str - containing source address for one address family.
                * Iterable of strings, containing multiple addresses
                    which to be distributed among the address families.
                    First address for given address family will be used.

        Returns:
            Dict of `address family` -> `source address`.
        """
        if not addr:
            return {}
        if isinstance(addr, str):
            return {Ping._get_afi(addr): addr}
        r: Dict[int, str] = {}
        for a in addr:
            afi = Ping._get_afi(a)
            if afi not in r:
                r[afi] = a
        return r

    def __get_socket(self: "Ping", address: str) -> PingSocket:
        """
        Get PingSocket instace.

        Get ping socket instance for specified address.
        Initialize when necessary.

        Args:
            address: Address to ping

        Returns:
            Initialized socket instance
        """
        afi = self._get_afi(address)
        sock = self.__sockets.get(afi)
        if not sock:
            sock = PingSocket(
                afi=afi,
                size=self.__size,
                src_addr=self.__src_addr.get(afi),
                ttl=self.__ttl if afi == IPv4 else None,
                tos=self.__tos if afi == IPv4 else None,
                timeout=self.__timeout,
                send_buffer_size=self.__send_buffer_size,
                recv_buffer_size=self.__recv_buffer_size,
                coarse=self.__coarse,
                accelerated=self.__accelerated,
            )
            self.__sockets[afi] = sock
        return sock

    def __get_request_id(self: "Ping") -> Tuple[int, int]:
        """
        Get request id.

        Generate ICMP request id and starting
        sequence number.

        Returns:
            Tuple of (`request_id`, `sequence`)
        """
        request_id = next(self.request_id) & 0xFFFF
        seq = random.randint(0, 0xFFFF)
        return request_id, seq

    async def ping(
        self: "Ping",
        addr: str,
        size: Optional[int] = None,
    ) -> Optional[float]:
        """
        Do ping probe.

        Send ICMP echo request to the given address and await
        for response or timeout.

        Args:
            addr: IPv4/IPv6 address to ping.
            size: Packet's size, including IP headers. Use PingSocket
                intialized defaults, when empty.

        Returns:
            * Round-trip time in seconds (as float) if success.
            * None - if failed or timed out.
        """
        sock = self.__get_socket(addr)
        request_id, seq = self.__get_request_id()
        return await sock.ping(addr, size=size, request_id=request_id, seq=seq)

    async def iter_rtt(
        self: "Ping",
        addr: str,
        *,
        size: Optional[int] = None,
        interval: Optional[float] = 1.0,
        count: Optional[int] = None,
    ) -> AsyncIterable[Optional[float]]:
        """
        Do the serie of ping probes.

        Send echo request every `interval` seconds,
        await and yield the result.

        Args:
            addr: Address to ping.
            size: Packets' size, including IP headers. Use PingSocket
                intialized defaults, when empty.
            interval: Interval between requests, in seconds.
            count: Stop after `count` requests, if set. Do not stop
                otherwise.

        Returns:
            Yields for each attempt:

            * Round-trip time in seconds (as float) if success.
            * None - if failed or timed out.

        """
        sock = self.__get_socket(addr)
        request_id, seq = self.__get_request_id()
        t0 = 0.0
        n = 0
        while True:
            if interval:
                t0 = perf_counter()
            yield await sock.ping(
                addr, size=size, request_id=request_id, seq=seq
            )
            seq = (seq + 1) & 0xFFFF
            if interval:
                dt = perf_counter() - t0
                if dt < interval:
                    await asyncio.sleep(interval - dt)
            n += 1
            if count and n >= count:
                break
