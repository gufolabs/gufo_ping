# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

To see unreleased changes, please see the [CHANGELOG on the main branch guide](https://github.com/gufolabs/gufo_ping/blob/main/CHANGELOG.md).

## [Unreleased]

### Changed

* docs: license.md renamed to LICENSE.md

### Infrastructure

* Adopt ruff
* Build Python 3.11 wheels for manylinux2014
* Rust 1.68.0
* PyO3 0.18
* socket2 0.5

## 0.2.4 - 2022-12-27

### Fixed

* Handle "No route to network/host" situation correctly

### Infrastructure

* Use `actions/checkout@v3`
* Use `actions/cache@v3`
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