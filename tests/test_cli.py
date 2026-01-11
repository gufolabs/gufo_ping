# ---------------------------------------------------------------------
# Gufo Ping: Test Ping
# ---------------------------------------------------------------------
# Copyright (C) 2022-26, Gufo Labs
# ---------------------------------------------------------------------

# Third-party modules
import pytest

from gufo.ping import SelectionPolicy

# Gufo Ping modules
from gufo.ping.cli import Cli, ExitCode, main


@pytest.mark.parametrize(
    "args", [["-c", "2", "127.0.0.1"], ["-c", "2", "192.0.2.1"]]
)
def test_cli(args: list[str]) -> None:
    r = main(args)
    assert r == ExitCode.OK


@pytest.mark.parametrize("args", [["-s", "10", "127.0.0.1"]])
def test_cli_error(args: list[str]) -> None:
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


@pytest.mark.parametrize("xclass", [NotImplementedError, PermissionError])
def test_probe_exceptions(xclass: type[BaseException]) -> None:
    class FaultyCli(Cli):
        async def _run(self, *args: str, **kwargs: str) -> ExitCode:
            raise xclass

    with pytest.raises(SystemExit):
        FaultyCli().run(["-c", "1", "127.0.0.1"])
