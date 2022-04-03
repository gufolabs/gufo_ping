from setuptools import setup
from setuptools_rust import RustExtension

setup(
    rust_extensions=[RustExtension("gufo.ping._fast")],
)
