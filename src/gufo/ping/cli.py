# ---------------------------------------------------------------------
# Gufo Ping: Command-line utility
# ---------------------------------------------------------------------
# Copyright (C) 2024-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""
`gufo-ping` command line utility.

Attributes:
    NAME: Utility's name.
"""

# Python modules
import argparse
import asyncio
import contextlib
import signal
import sys
from enum import IntEnum
from typing import List, NoReturn, Optional

# Gufo Ping modules
from gufo.ping import Ping

NAME = "gufo-ping"
MIN_SIZE = 64


class ExitCode(IntEnum):
    """
    Cli exit codes.

    Attributes:
        OK: Successful exit
    """

    OK = 0
    ERR = 1


class Cli(object):
    """`gufo-ping` utility class."""

    @classmethod
    def die(cls, msg: Optional[str] = None) -> NoReturn:
        """Die with message."""
        if msg:
            print(msg)
        sys.exit(1)

    def run(self, args: List[str]) -> ExitCode:
        """
        Parse command-line arguments and run appropriate command.

        Args:
            args: List of command-line arguments
        Returns:
            ExitCode
        """
        # Prepare command-line parser
        parser = argparse.ArgumentParser(prog=NAME, description="HTTP Client")
        parser.add_argument("address", nargs=1, help="Address")
        parser.add_argument(
            "-c",
            "--count",
            type=int,
            help="Stop after sending and receiving `count` packets",
        )
        parser.add_argument(
            "-s",
            "--size",
            type=int,
            help="Packet size",
        )
        # Parse arguments
        ns = parser.parse_args(args)
        if ns.size is not None and ns.size < MIN_SIZE:
            self.die(f"size must be more than {MIN_SIZE}")
        # Setup loop
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        main_task = loop.create_task(
            self._run(ns.address[0], count=ns.count, size=ns.size)
        )
        for sig in (signal.SIGINT, signal.SIGTERM):
            loop.add_signal_handler(sig, main_task.cancel)
        # Run
        try:
            return loop.run_until_complete(main_task)
        finally:
            loop.close()

    async def _run(
        self,
        /,
        address: str,
        count: Optional[int] = None,
        size: Optional[int] = None,
    ) -> ExitCode:
        size = size or MIN_SIZE
        ping = Ping(size=size)
        n = 0
        sent = 0
        received = 0
        print(f"PING {address}: {size} bytes")
        with contextlib.suppress(asyncio.CancelledError):
            async for r in ping.iter_rtt(addr=address):
                sent += 1
                if r is None:
                    print(f"Request timeout for icmp_seq {n}")
                else:
                    print(
                        f"{size} bytes from {address}: "
                        f"icmp_seq={n} time={r * 1000.0:.3f}ms"
                    )
                    received += 1
                n += 1
                if count is not None and n >= count:
                    break
        print(f"--- {address} ping statistics ---")
        loss = float(sent - received) / float(sent) * 100.0
        print(
            f"{sent} packets transmitted, "
            f"{received} packets received, "
            f"{loss:.1f}% packet loss"
        )
        return ExitCode.OK


def main(args: Optional[List[str]] = None) -> int:
    """Run `gufo-ping` with command-line arguments."""
    return Cli().run(sys.argv[1:] if args is None else args).value
