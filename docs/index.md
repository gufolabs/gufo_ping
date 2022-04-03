# Gufo Labs Ping Documentation

*Gufo Labs Ping* is the accelerated Python asyncio IPv4/IPv6 ping implementation.

## The Ping Problem

A common way of probing host availability over IP networks is to use ICMP echo probes. Almost every OS is packaged with the famous ping utility. But what if you need to embed
the probe just into your Python application? Running system `ping` is rarely the option. System ping output formats may vary from system to system. Maintaining the application quickly became a road to hell.
Embedding a ping into your application may be a better choice.

Despite the apparent simplicity, it is not a trivial task. To implement the ping you have to master a system-dependent Raw Sockets magic. Next, you have to implement ICMP packet forging and parsing. There are many pitfalls. Obtaining a high performance is a separate challenge.

## Gufo Ping

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