import sys
import asyncio
from gufo.ping import Ping


async def main(addr: str) -> None:
    ping = Ping()
    r = await ping.ping(addr)
    print(r)


if __name__ == "__main__":
    asyncio.run(main(sys.argv[1]))
