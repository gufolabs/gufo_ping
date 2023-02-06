# ---------------------------------------------------------------------
# Gufo Ping: Test Utilities
# ---------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# ---------------------------------------------------------------------

# Python modules
import socket
from typing import Optional


def is_denied() -> bool:
    """
    Check if raw sockets is denied.

    Returns:
        * True - if raw sockets are denied.
        * False - otherwise.
    """
    global _DENIED

    if _DENIED is None:
        try:
            socket.socket(socket.AF_INET, socket.SOCK_RAW, socket.IPPROTO_ICMP)
            _DENIED = False
        except OSError:
            _DENIED = True
    return _DENIED


_DENIED: Optional[bool] = None
