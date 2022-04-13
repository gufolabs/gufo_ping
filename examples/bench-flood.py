import sys
import time
import asyncio
from gufo.ping import Ping


async def main(address: str, size: int = 64, count: int = 4) -> None:
    ping = Ping(size=size)
    t0 = time.time()
    print(f"PING {address}: {size} bytes, {count} packets")
    received = 0
    async for r in ping.iter_rtt(address, count=count, interval=None):
        if r is not None:
            received += 1
    print(f"--- {address} ping statistics ---")
    loss = float(count - received) / float(count) * 100.0
    duration = time.time() - t0
    print(
        f"{count} packets transmitted, "
        f"{received} packets received, "
        f"{loss:.1f}% packet loss"
    )
    print(f"{duration:.3f}s, {count / duration:.1f}req/sec")


if __name__ == "__main__":
    asyncio.run(main(sys.argv[1], count=10_000))
