import asyncio
import sys

from gufo.ping import Ping


async def main(addr: str) -> None:
    ping = Ping()
    async for r in ping.iter_rtt(addr, count=5):
        print(r)


if __name__ == "__main__":
    asyncio.run(main(sys.argv[1]))
