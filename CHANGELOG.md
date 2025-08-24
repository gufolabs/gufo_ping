---
hide:
    - navigation
---
# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

To see unreleased changes, please see the [CHANGELOG on the main branch guide](https://github.com/gufolabs/gufo_ping/blob/main/CHANGELOG.md).

## 0.5.1 - 2025-08-24

### Security

* CVE-2023-24816: Bump IPython version

### Infrastructure

* IPython 9.4.0
* ruff 0.11.2

### Removed

* Python 3.8 support

## 0.5.0 - 2025-03-12

### Added

* ARM64 binary wheels

### Infrastructure

* Rust 1.85.0
* Rust edition 2024

## 0.4.0 - 2025-01-29

### Added

* Python 3.13 support

### Fixed

* [#2][#2] BUG: Error setting DSCP/ToS field for IPv6 ICMP packets
* [#4][#4] BUG: Setting ttl fails for IPv6

### Removed

* Python 3.8 support

### Infrastructure

* Use Python 1.13 in devcontainer
* Use Ruff for formatting
* Rust 1.84.0
* PyO3 0.23
* mkdocs-material 9.5.44
* Ruff 0.7.2
* mypy 1.13.0
* pytest 8.3.3
* Coverage 7.6.4
* Bump GitHub actions

## 0.3.1 -- 2023-12-11

### Added

* Python 3.12 support.
* docs: Fancy front page

### Changed

* Use `manylinux_2_28` instead of `manylinux_2_24`.

### Infrastructure

* devcontainer: YAML formatting on save
* Rust 1.74.1
* devcontainer: Use Python 3.12
* byteorder 1.5
* pyo3 0.20

## 0.3.0 -- 2023-05-02

### Added

* Ping `src_addr` parameter to set source addresses of outgoing packets.

### Changed

* Ping's `ttl` and `tos` options are temporary ignored on IPv6
* docs: license.md renamed to LICENSE.md

### Infrastructure

* Adopt ruff
* Build Python 3.11 wheels for manylinux2014
* Rust 1.69
* PyO3 0.18
* socket2 0.5
* devcontainer: Move `settings` to `customizations.vscode.settings`

## 0.2.4 - 2022-12-27

### Fixed

* Handle "No route to network/host" situation correctly

### Infrastructure

* Use `actions/checkout@v4`
* Use `actions/cache@v4`
* Project structure tests
* CI workflows tests
* Rust 1.66.0

## 0.2.3 - 2022-11-17

### Added

* Python 3.11 compatibility
* `py.typed` file for PEP-561 compatibility
* Add CITATION.cff

### Changed

* Reworked wheels builder
* Move CHANGELOG.md to the project root
* Rename `_fast.py` to `_fast.pyi`

### Infrastructure

* Rust 1.65.0
* PyO3 0.17
* setuptools-rust 1.5.2
* mkdocs-material 0.8.5
* Unify Rust setup for Dockerfile and GitHub CI
* Use Python 3.11 in Devcontainer

## 0.2.2 - 2022-05-16

### Infrastructure

* Rollback to `manylinux2014` for RHEL7 compatibility.
* PyO3 0.16.4

## 0.2.1 - 2022-04-15

### Added

* `__version__` attribute.

## 0.2.0 - 2022-04-14

### Added

* examples/bench-flood.py script.

### Changed

* Optimized buffer handling (~10% speedup).
* Apply BPF filter to raw socket to reduce context switches (Linux).

### Infrastructure

* Switch to Rust 1.60.0.

## 0.1.0 - 2022-04-11

### Added

* Initial release.

[#2]: https://github.com/gufolabs/gufo_ping/issues/2
[#4]: https://github.com/gufolabs/gufo_ping/issues/4