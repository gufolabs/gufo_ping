# ---------------------------------------------------------------------
# Gufo Ping: ICMPv4/ICMPv6 ping implementation
# ---------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# ---------------------------------------------------------------------

"""
Attributes:
    __version__: Current version
"""

# Gufo Labs modules
from .ping import Ping  # noqa

__version__: str = "0.2.2"
__all__ = ["Ping", "__version__"]
