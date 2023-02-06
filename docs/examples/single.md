# Gufo Ping Example: Single Request

Lets send the single ICMP echo request and get the
reply.

```  py title="single.py" linenums="1"
--8<-- "examples/single.py"
```

The code is straightforward:

```  py title="single.py" linenums="1" hl_lines="1"
--8<-- "examples/single.py"
```

We need `asyncio.run()` to run asynchronous code, so lets import the `asyncio`.

```  py title="single.py" linenums="1" hl_lines="2"
--8<-- "examples/single.py"
```

Import `sys` module to parse the CLI argument.

!!! warning

    We use `sys.argv` only for demonstration purposes. Use `argsparse` or alternatives
    in real-world applications.


```  py title="single.py" linenums="1" hl_lines="4"
--8<-- "examples/single.py"
```

`Ping` object holds all necessary API, so import it from `gufo.ping`.

```  py title="single.py" linenums="1" hl_lines="7"
--8<-- "examples/single.py"
```

Asynchronous code must be executed in the asynchronous functions, or coroutines.
So we define our function as `async`. We expect an address to ping as the
`addr` argument.

```  py title="single.py" linenums="1" hl_lines="8"
--8<-- "examples/single.py"
```

First we need to create `Ping` object. `Ping` constructor offers a lots
of configuration variables for fine-tuning. Refer to the 
[Ping reference](../../reference/gufo/ping/ping#gufo.ping.ping.Ping)
for further details. Defaults are good enough for our tutorial, so
we ommited them.

```  py title="single.py" linenums="1" hl_lines="9"
--8<-- "examples/single.py"
```

Run single ping probe. `Ping.ping` is asynchronous function, so note
the `await` keyword. The only mandatory parameter is IP address.
Gufo Ping detects IPv4/IPv6 usage, so all we need is to pass an address.
Function may accept the additional parameters for fine-tuning,
Refer to the
[Ping.ping reference](../../reference/gufo/ping/ping#gufo.ping.ping.Ping.ping)
for details.

Ping returns:

* In case of success: Round-trip-time (RTT), as float, measured in seconds.
* In case of failure: `None`.

```  py title="single.py" linenums="1" hl_lines="10"
--8<-- "examples/single.py"
```

Lets print the result for our example.

```  py title="single.py" linenums="1" hl_lines="13"
--8<-- "examples/single.py"
```

To run our example from command line we need to check
we're in `__main__` module.

```  py title="single.py" linenums="1" hl_lines="14"
--8<-- "examples/single.py"
```

Lets run our asyncronous `main()` function via `asyncio.run`
and pass first command-line parameter as address.

## Running

Lets check the success case. Run example as:

```
$ sudo python3 examples/single.py 127.0.0.1
0.000973377
```

Ok, we got a measured RTT.

Lets check the faulty cause. [RFC-5737][RFC-5737] defines the special range
of addresses - 192.0.2.0/24, which can be used as unresponsible addresses.
So, lets ping a 192.0.2.1:

```
$ sudo python3 examples/single.py 192.0.2.1
None
```

After second or so of awaiting we finally got a `None`, which means our
request is timed out.

[RFC-5737]: https://datatracker.ietf.org/doc/html/rfc5737