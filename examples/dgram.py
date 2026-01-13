import asyncio
import sys

from gufo.ping import Ping, SelectionPolicy


async def main(addr: str) -> None:
    ping = Ping(policy=SelectionPolicy.DGRAM)
    r = await ping.ping(addr)
    print(r)


if __name__ == "__main__":
    asyncio.run(main(sys.argv[1]))
