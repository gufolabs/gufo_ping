# Gufo Labs Ping

*Gufo Labs Ping is the accelerated Python asyncio IPv4/IPv6 ping implementation.*

[![PyPi version](https://img.shields.io/pypi/v/gufo_ping.svg)](https://pypi.python.org/pypi/gufo_ping/)
![Python Versions](https://img.shields.io/pypi/pyversions/gufo_ping)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
![Build](https://img.shields.io/github/workflow/status/gufolabs/gufo_ping/Run%20Tests/master)
![Sponsors](https://img.shields.io/github/sponsors/gufolabs)

---

**Documentation**: [https://docs.gufolabs.com/gufo_ping/](https://docs.gufolabs.com/gufo_ping/)

**Source Code**: [https://github.com/gufolabs/gufo_ping/](https://github.com/gufolabs/gufo_ping/)

---

Gufo Ping is the Python asyncio library for IPv4/IPv6 ping probing. It consists of a clean Python API for high-efficient raw sockets manipulation, implemented in the 
[Rust][Rust] language with [PyO3][PyO3] wrapper.

Pinging host is the simple task:

``` py
ping = Ping()
rtt = await ping.ping("127.0.0.1")
```

Sending the series of probes is simple too:

``` py
ping = Ping()
async for rtt in ping.iter_rtt("127.0.0.1", count=5):
    print(rtt)
```

Gufo Ping is really fast, allowing to probe 100 000+ hosts at once.

## Virtues

* Clean async API.
* IPv4/IPv6 support.
* High-performance.
* Full Python typing support.
* Editor completion.
* Well-tested, battle-proven code.

[Rust]: https://rust-lang.org/
[PyO3]: https://pyo3.rs/