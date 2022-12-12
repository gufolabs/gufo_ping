# ---------------------------------------------------------------------
# Gufo Labs: Project structure tests
# ---------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import os

# Third-party modules
import pytest


def _get_project():
    d = [
        f
        for f in os.listdir(os.path.join("src", "gufo"))
        if not f.startswith(".") and not f.startswith("_")
    ]
    assert len(d) == 1
    return d[0]


PROJECT = _get_project()

REQUIRED_FILES = [
    ".devcontainer/devcontainer.json",
    ".github/CODEOWNERS",
    ".github/ISSUE_TEMPLATE/bug-report.yml",
    ".github/ISSUE_TEMPLATE/feature-request.yml",
    ".github/ISSUE_TEMPLATE/security-report.yml",
    ".github/PULL_REQUEST_TEMPLATE.md",
    ".gitignore",
    ".requirements/docs.txt",
    ".requirements/lint.txt",
    ".requirements/test.txt",
    "CHANGELOG.md",
    "CITATION.cff",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "Dockerfile",
    "LICENSE.md",
    "README.md",
    "SECURITY.md",
    "docs/assets/logo.png",
    "docs/codebase.md",
    "docs/codequality.md",
    "docs/devcommon.md",
    "docs/environment.md",
    "docs/examples/index.md",
    "docs/faq.md",
    "docs/index.md",
    "docs/installation.md",
    "docs/testing.md",
    "mkdocs.yml",
    "pyproject.toml",
    "setup.cfg",
    f"src/gufo/{PROJECT}/__init__.py",
    f"src/gufo/{PROJECT}/py.typed",
    "tests/test_docs.py",
    "tests/test_project.py",
]


def test_required_is_sorted():
    assert REQUIRED_FILES == list(
        sorted(REQUIRED_FILES)
    ), "REQUIRED_FILES must be sorted"


@pytest.mark.parametrize("name", REQUIRED_FILES)
def test_required_files(name: str):
    assert os.path.exists(name), f"File {name} is missed"


def test_version():
    m = __import__(f"gufo.{PROJECT}", {}, {}, "*")
    assert hasattr(m, "__version__"), "__init__.py must contain __version__"
