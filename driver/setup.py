#!/usr/bin/env python
import sys

from setuptools import setup

try:
    from setuptools_rust import RustExtension
except ImportError:
    import subprocess

    errno = subprocess.call([sys.executable, "-m", "pip", "install", "setuptools-rust"])
    if errno:
        print("Please install setuptools-rust package")
        raise SystemExit(errno)
    else:
        from setuptools_rust import RustExtension

setup_requires = ["setuptools-rust>=0.10.1", "wheel"]
install_requires = []

with open("README.md", "r") as fh:
    long_description = fh.read()

setup(
    name="zscanfd",
    version="0.2.0",
    author="Zilogic Systems",
    author_email="code@zilogic.com",
    description="Support for the zs-canfd Controller Area Network Devices",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/zilogic-systems/zscanfd-driver",
    classifiers=[
        "License :: OSI Approved :: MIT License",
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Programming Language :: Python",
        "Programming Language :: Rust",
        "Operating System :: POSIX :: Linux",
        "Operating System :: MacOS :: MacOS X",
        "Operating System :: Microsoft :: Windows",
        "Topic :: System :: Hardware :: Hardware Drivers",
        "Topic :: System :: Networking",
        "Topic :: Software Development :: Embedded Systems",
    ],
    packages=["zscanfd"],
    rust_extensions=[RustExtension("zscanfd.zscanfd", features=["python"],)],
    install_requires=install_requires,
    setup_requires=setup_requires,
    python_requires=">=3.5",
    include_package_data=True,
    zip_safe=False,
)
