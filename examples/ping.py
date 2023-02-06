import asyncio
import sys

from gufo.ping import Ping


async def main(address: str, size: int = 64, count: int = 4) -> None:
    ping = Ping(size=size)
    n = 0
    print(f"PING {address}: {size} bytes")
    received = 0
    async for r in ping.iter_rtt(address):
        if r is None:
            print(f"Request timeout for icmp_seq {n}")
        else:
            print(
                f"{size} bytes from {address}: "
                f"icmp_seq={n} time={r*1000.0:.3f}ms"
            )
            received += 1
        n += 1
        if n >= count:
            break
    print(f"--- {address} ping statistics ---")
    loss = float(count - received) / float(count) * 100.0
    print(
        f"{count} packets transmitted, "
        f"{received} packets received, "
        f"{loss:.1f}% packet loss"
    )


if __name__ == "__main__":
    asyncio.run(main(sys.argv[1]))
