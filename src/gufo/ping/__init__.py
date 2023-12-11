# ---------------------------------------------------------------------
# Gufo Ping: ICMPv4/ICMPv6 ping implementation
# ---------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# ---------------------------------------------------------------------

"""
Gufo Ping is the accelerated Python asyncio IPv4/IPv6 ping implementation.

Attributes:
    __version__: Current version.
"""

# Gufo Labs modules
from .ping import Ping

__version__: str = "0.3.1"
__all__ = ["Ping", "__version__"]
