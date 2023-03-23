# ---------------------------------------------------------------------
# Gufo Ping: Test Utilities
# ---------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# ---------------------------------------------------------------------

# Python modules
import socket
from functools import cached_property
from typing import Any, Dict, List


class Caps(object):
    @cached_property
    def has_ipv4(self: "Caps") -> bool:
        """
        Check system allows IPv4 raw sockets.

        Returns:
            * True - if IPv4 raw sockets are allowed.
            * False - if IPv4 raw sockets are denied.
        """
        try:
            s = socket.socket(
                socket.AF_INET, socket.SOCK_RAW, socket.IPPROTO_ICMP
            )
            s.bind(("127.0.0.1", 0))
            return True
        except OSError:
            return False

    @cached_property
    def has_ipv6(self: "Caps") -> bool:
        """
        Check system allows IPv6 raw sockets.

        Returns:
            * True - if IPv6 raw sockets are allowed.
            * False - if IPv6 raw sockets are denied.
        """
        try:
            s = socket.socket(
                socket.AF_INET6, socket.SOCK_RAW, socket.IPPROTO_ICMPV6
            )
            s.bind(("::1", 0))
            return True
        except OSError:
            return False

    @cached_property
    def is_denied(self: "Caps") -> bool:
        """Check if all raw sockets are denied."""
        return not (self.has_ipv4 or self.has_ipv6)

    @cached_property
    def loopbacks(self: "Caps") -> List[str]:
        """
        Get list of loopback addresses.

        Returns:
            List of IPv4/IPv6 loopbback addresses for all
            allowed protocols. Empty if raw sockets are
            denied.
        """
        r: List[str] = []
        if self.has_ipv4:
            r.append("127.0.0.1")
        if self.has_ipv6:
            r.append("::1")
        return r


def as_str(v: Dict[str, Any]) -> str:
    """
    Format parameters for @parametrize(..., ids).

    Args:
        v: Input parameters.

    Returns:
        String to display as test id.

    Example:
        ``` py
        @pytest.mark.parametrize(...., ids=as_str)
        ```
    """
    return str(v)


caps = Caps()
