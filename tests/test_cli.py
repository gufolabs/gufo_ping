# ---------------------------------------------------------------------
# Gufo Ping: Test Ping
# ---------------------------------------------------------------------
# Copyright (C) 2022-25, Gufo Labs
# ---------------------------------------------------------------------

# Python modules
from typing import List

# Third-party modules
import pytest

# Gufo Ping modules
from gufo.ping.cli import Cli, ExitCode


@pytest.mark.parametrize(
    "args", [["-c", "2", "127.0.0.1"], ["-c", "2", "192.0.2.1"]]
)
def test_cli(args: List[str]) -> None:
    r = Cli().run(args)
    assert r == ExitCode.OK
