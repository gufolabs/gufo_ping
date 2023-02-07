# Gufo Ping

*Gufo Ping is the accelerated Python asyncio IPv4/IPv6 ping implementation.*

[![PyPi version](https://img.shields.io/pypi/v/gufo_ping.svg)](https://pypi.python.org/pypi/gufo_ping/)
![Python Versions](https://img.shields.io/pypi/pyversions/gufo_ping)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
![Build](https://img.shields.io/github/actions/workflow/status/gufolabs/gufo_ping/tests.yml?branch=master)
![Sponsors](https://img.shields.io/github/sponsors/gufolabs)
[![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/charliermarsh/ruff/main/assets/badge/v0.json)](https://github.com/charliermarsh/ruff)

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

## On Gufo Stack

This product is a part of [Gufo Stack][Gufo Stack] - the collaborative effort 
led by [Gufo Labs][Gufo Labs]. Our goal is to create a robust and flexible 
set of tools to create network management software and automate 
routine administration tasks.

To do this, we extract the key technologies that have proven themselves 
in the [NOC][NOC] and bring them as separate packages. Then we work on API,
performance tuning, documentation, and testing. The [NOC][NOC] uses the final result
as the external dependencies.

[Gufo Stack][Gufo Stack] makes the [NOC][NOC] better, and this is our primary task. But other products
can benefit from [Gufo Stack][Gufo Stack] too. So we believe that our effort will make 
the other network management products better.

[Gufo Labs]: https://gufolabs.com/
[Gufo Stack]: https://gufolabs.com/products/gufo-stack/
[NOC]: https://getnoc.com/
[Rust]: https://rust-lang.org/
[PyO3]: https://pyo3.rs/
