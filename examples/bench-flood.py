import asyncio
import sys
import time

from gufo.ping import Ping, SelectionPolicy


async def main(
    address: str,
    size: int = 64,
    count: int = 4,
    policy: SelectionPolicy | None = None,
) -> None:
    ping = Ping(size=size, policy=policy or SelectionPolicy.RAW)
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
    import argparse

    POLICY_MAP = {
        "raw": SelectionPolicy.RAW,
        "dgram": SelectionPolicy.DGRAM,
        "raw,dgram": SelectionPolicy.RAW_DGRAM,
        "dgram,raw": SelectionPolicy.DGRAM_RAW,
    }
    parser = argparse.ArgumentParser(
        prog="bench-flood", description="Flood ping benchmark"
    )
    parser.add_argument(
        "-p",
        "--policy",
        choices=list(POLICY_MAP),
        default="dgram,raw",
        help="Probe selection policy",
    )
    parser.add_argument(
        "-c", "--count", type=int, default=100_000, help="Packets count"
    )
    parser.add_argument("address", nargs=1, help="Address")
    ns = parser.parse_args()
    asyncio.run(
        main(ns.address[0], count=ns.count, policy=POLICY_MAP[ns.policy])
    )
