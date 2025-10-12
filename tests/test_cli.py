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
from gufo.ping.cli import ExitCode, main


@pytest.mark.parametrize(
    "args", [["-c", "2", "127.0.0.1"], ["-c", "2", "192.0.2.1"]]
)
def test_cli(args: List[str]) -> None:
    r = main(args)
    assert r == ExitCode.OK


@pytest.mark.parametrize("args", [["-s", "10"]])
def test_cli_error(args: List[str]) -> None:
    with pytest.raises(SystemExit):
        main(args)
