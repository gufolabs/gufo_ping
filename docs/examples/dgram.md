# Gufo Ping Example: Using DGRAM socket.

Gufo Ping uses RAW sockets by default, which is the traditional approach for implementing ping.
A common drawback of this method is that the process must run as the root user or, on Linux platforms,
have the `CAP_NET_RAW` capability.

Even when only `CAP_NET_RAW` is granted, the process gains the ability to intercept all host traffic
and to forge arbitrary outgoing packets. This makes any pinging process a high-value target for potential attacks.

Fortunately, Linux offers an alternative approach: using `DGRAM` sockets for ICMP echo requests.
In this mode, the process’s group ID must fall within the range defined
by the `net.ipv4.ping_group_range` sysctl. With this restriction in place,
the process can send only ICMP echo requests and receive only replies to its own requests,
which greatly reduces the security impact.

Let us send a single ICMP echo request and receive the reply using a `DGRAM` socket.

```  py title="dgram.py" linenums="1"
--8<-- "examples/dgram.py"
```

The code is straightforward:

```  py title="dgram.py" linenums="1" hl_lines="1"
--8<-- "examples/dgram.py"
```

We need `asyncio.run()` to run asynchronous code, so lets import the `asyncio`.

```  py title="dgram.py" linenums="1" hl_lines="2"
--8<-- "examples/dgram.py"
```

Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.


```  py title="dgram.py" linenums="1" hl_lines="4"
--8<-- "examples/dgram.py"
```

`Ping` object holds all necessary API, so import it from `gufo.ping`. We also need `SelectionPolicy`
enum to select desired probing method.

```  py title="dgram.py" linenums="1" hl_lines="7"
--8<-- "examples/dgram.py"
```

Asynchronous code must be executed in the asynchronous functions, or coroutines.
So we define our function as `async`. We expect an address to ping as the
`addr` argument.

```  py title="dgram.py" linenums="1" hl_lines="8"
--8<-- "examples/dgram.py"
```

First we need to create `Ping` object. `Ping` constructor offers a lots
of configuration variables for fine-tuning. We're setting `policy` parameter
which is responsible for the probing method selection. `SelectionPolicy.DGRAM`
switches to the `DGRAM` socket mode. Refer to the
[SelectionPolicy reference](http://127.0.0.1:8001/gufo_ping/reference/gufo/ping/socket/#gufo.ping.socket.SelectionPolicy)
for possible values.

```  py title="dgram.py" linenums="1" hl_lines="9"
--8<-- "examples/dgram.py"
```

Run single ping probe. `Ping.ping` is asynchronous function, so note
the `await` keyword. The only mandatory parameter is IP address.
Gufo Ping detects IPv4/IPv6 usage, so all we need is to pass an address.
Function may accept the additional parameters for fine-tuning,
Refer to the
[Ping.ping reference](../reference/gufo/ping/ping.md#gufo.ping.ping.Ping.ping)
for details.

Ping returns:

* In case of success: Round-trip-time (RTT), as float, measured in seconds.
* In case of failure: `None`.

```  py title="dgram.py" linenums="1" hl_lines="10"
--8<-- "examples/dgram.py"
```

Lets print the result for our example.

```  py title="dgram.py" linenums="1" hl_lines="13"
--8<-- "examples/dgram.py"
```

To run our example from command line we need to check
we're in `__main__` module.

```  py title="dgram.py" linenums="1" hl_lines="14"
--8<-- "examples/dgram.py"
```

Lets run our asyncronous `main()` function via `asyncio.run`
and pass first command-line parameter as address.

## Running

Lets check the success case. Run example as:

```
$ python3 examples/dgram.py 127.0.0.1
0.000973377
```

Ok, we got a measured RTT.

Lets check the faulty cause. [RFC-5737][RFC-5737] defines the special range
of addresses - 192.0.2.0/24, which can be used as unresponsible addresses.
So, lets ping a 192.0.2.1:

```
$ python3 examples/dgram.py 192.0.2.1
None
```

After second or so of awaiting we finally got a `None`, which means our
request is timed out.

[RFC-5737]: https://www.rfc-editor.org/rfc/rfc5737.html