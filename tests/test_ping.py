# ---------------------------------------------------------------------
# Gufo Ping: Test Ping
# ---------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# ---------------------------------------------------------------------

# Python modules
import asyncio
from typing import Any, Dict, Iterable, List, Optional, Union

# Third-party modules
import pytest

# Gufo Labs modules
from gufo.ping import Ping

from .util import as_str, caps


@pytest.mark.parametrize(
    ("address", "expected"), [("127.0.0.1", 4), ("::1", 6)]
)
def test_get_afi(address: str, expected: int) -> None:
    afi = Ping._get_afi(address)
    assert afi == expected


@pytest.mark.skipif(caps.is_denied, reason="Permission denied")
@pytest.mark.parametrize(
    ("address", "expected"),
    [
        # IPv4
        ("127.0.0.1", True),  # Loopback, always available
        ("192.0.2.1", False),  # RFC-5737 test range, should fail
    ],
)
def test_ping(address: str, expected: bool) -> None:
    ping = Ping()
    rtt = asyncio.run(ping.ping(address))
    if expected:
        assert isinstance(rtt, float)
        assert rtt > 0.0
    else:
        assert rtt is None


@pytest.mark.skipif(caps.is_denied, reason="Permission denied")
@pytest.mark.parametrize(
    ("address", "expected"),
    [
        # IPv4
        ("127.0.0.1", True),  # Loopback, always available
        ("192.0.2.1", False),  # RFC-5737 test range, should fail
    ],
)
def test_iter_rtt(address: str, expected: bool) -> None:
    async def inner() -> List[Optional[float]]:
        r: List[Optional[float]] = []
        async for rtt in ping.iter_rtt(address, count=N_PROBES):
            r.append(rtt)
        return r

    N_PROBES = 5
    ping = Ping()
    res = asyncio.run(inner())
    assert len(res) == N_PROBES
    if expected:
        nr = sum(1 for rtt in res if rtt is not None)
        assert nr == N_PROBES
    else:
        nr = sum(1 for rtt in res if rtt is None)
        assert nr == N_PROBES


@pytest.mark.skipif(caps.is_denied, reason="Permission denied")
@pytest.mark.parametrize("addr", caps.loopbacks)
@pytest.mark.parametrize(
    ("cfg", "expected"),
    [
        # ttl
        ({"ttl": -1}, False),
        ({"ttl": 0}, False),
        ({"ttl": 1}, True),
        ({"ttl": 64}, True),
        ({"ttl": 255}, True),
        ({"ttl": 256}, False),
        # tos
        ({"tos": -1}, False),
        ({"tos": 0}, True),
        ({"tos": 1}, True),
        ({"tos": 64}, True),
        ({"tos": 255}, True),
        ({"tos": 256}, False),
        # send_buffer_size
        ({"send_buffer_size": 1048576}, True),
        # recv_buffer_size
        ({"recv_buffer_size": 1048576}, True),
        # coarse
        ({"coarse": True}, True),
        ({"coarse": False}, True),
    ],
    ids=as_str,
)
def test_valid_ping_settings(
    addr: str, cfg: Dict[str, Any], expected: bool
) -> None:
    if expected:
        asyncio.run(Ping(**cfg).ping(addr))
    else:
        with pytest.raises(ValueError):
            asyncio.run(Ping(**cfg).ping(addr))


@pytest.mark.parametrize(
    ("src_addr", "expected"),
    [
        (None, {}),
        ("127.0.0.1", {4: "127.0.0.1"}),
        ("::1", {6: "::1"}),
        ((), {}),
        ([], {}),
        (("127.0.0.1", "::1"), {4: "127.0.0.1", 6: "::1"}),
        (["127.0.0.1", "::1"], {4: "127.0.0.1", 6: "::1"}),
        (("::1", "127.0.0.1"), {4: "127.0.0.1", 6: "::1"}),
        (("127.0.0.1", "127.0.0.2"), {4: "127.0.0.1"}),
        (("127.0.0.2", "127.0.0.1", "::1", "::2"), {4: "127.0.0.2", 6: "::1"}),
    ],
    ids=as_str,
)
def test_src_addr(
    src_addr: Union[None, str, Iterable[str]], expected: Dict[int, str]
) -> None:
    assert Ping._get_src_addr(src_addr) == expected


@pytest.mark.skipif(caps.is_denied, reason="Permission denied")
@pytest.mark.parametrize("addr", caps.loopbacks)
def test_src_ping(addr: str) -> None:
    ping = Ping(src_addr=addr)
    rtt = asyncio.run(ping.ping(addr))
    assert isinstance(rtt, float)
    assert rtt > 0.0
