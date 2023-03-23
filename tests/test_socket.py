# ---------------------------------------------------------------------
# Gufo Ping: PingSocket test
# ---------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# ---------------------------------------------------------------------

# Python modules
import asyncio

# Third-party modules
import pytest

# Gufo Labs modules
from gufo.ping.socket import PingSocket

from .util import caps


@pytest.mark.skipif(caps.is_denied, reason="Permission denied")
@pytest.mark.parametrize(
    ("afi", "expected"), [(1, False), (4, True), (6, True)]
)
def test_afi(afi: int, expected: bool) -> None:
    async def inner_fail() -> None:
        with pytest.raises(ValueError):
            PingSocket(afi=afi)

    async def inner_ok() -> None:
        PingSocket(afi=afi)

    asyncio.run(inner_ok() if expected else inner_fail())


@pytest.mark.skipif(caps.is_denied, reason="Permission denied")
def test_empty_read() -> None:
    async def inner() -> None:
        s = PingSocket(afi=4)
        r = s._on_read()
        assert r is None

    asyncio.run(inner())


@pytest.mark.skipif(caps.is_denied, reason="Permission denied")
@pytest.mark.parametrize(
    ("afi", "addr", "expected"),
    [
        #  IPv4
        (4, "127.0.0.1", "127.0.0.1"),
        (4, "127.0.1", None),
        (4, "127.0.0.01", None),
        # IPv6
        (6, "::1", "::1"),
        (6, "0::1", "::1"),
    ],
)
def test_clean_ip(afi: int, addr: str, expected: str) -> None:
    async def inner_ok() -> None:
        s = PingSocket(afi=afi)
        assert s.clean_ip(addr) == expected

    async def inner_fail() -> None:
        s = PingSocket(afi=afi)
        with pytest.raises(ValueError):
            s.clean_ip(addr)

    asyncio.run(inner_ok() if expected else inner_fail())
