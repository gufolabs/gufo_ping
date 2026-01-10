# ---------------------------------------------------------------------
# Gufo Ping: Test Ping
# ---------------------------------------------------------------------
# Copyright (C) 2022-25, Gufo Labs
# ---------------------------------------------------------------------

# Python modules
from typing import List

# Third-party modules
import pytest

from gufo.ping import SelectionPolicy

# Gufo Ping modules
from gufo.ping.cli import Cli, ExitCode, main


@pytest.mark.parametrize(
    "args", [["-c", "2", "127.0.0.1"], ["-c", "2", "192.0.2.1"]]
)
def test_cli(args: List[str]) -> None:
    r = main(args)
    assert r == ExitCode.OK


@pytest.mark.parametrize("args", [["-s", "10", "127.0.0.1"]])
def test_cli_error(args: List[str]) -> None:
    with pytest.raises(SystemExit):
        main(args)


def test_cli_die() -> None:
    with pytest.raises(SystemExit):
        Cli.die("die!")


@pytest.mark.parametrize(
    ("v", "expected"),
    [
        ("raw", SelectionPolicy.RAW),
        ("raw,dgram", SelectionPolicy.RAW_DGRAM),
        ("dgram,raw", SelectionPolicy.DGRAM_RAW),
        ("dgram", SelectionPolicy.DGRAM),
    ],
)
def test_policy(v: str, expected: SelectionPolicy) -> None:
    assert Cli._get_policy(v) == expected
