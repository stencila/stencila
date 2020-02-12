#!/usr/bin/env python
# -*- coding: utf-8 -*-

import sys

import io
import os
import subprocess
from setuptools import setup, Command
from shutil import rmtree

HERE = os.path.abspath(os.path.dirname(__file__))

with io.open(os.path.join(HERE, "README.md"), encoding="utf-8") as f:
    long_description = "\n" + f.read()

VERSION_PATH = os.path.join(HERE, "version")


def get_tag_version() -> str:
    """Get the version by parsing git tag."""
    result = subprocess.run(
        ["git", "describe", "--tags", "--abbrev=0"],
        stdout=subprocess.PIPE,
        encoding="ascii",
    )
    version = result.stdout
    return version[1:] if version.startswith("v") else version


def get_version():
    """On build (in CI), the version should be written out, then included and be available during install."""
    if os.path.exists(VERSION_PATH):
        with open(VERSION_PATH) as vf:
            return vf.read().strip()

    tag_version = get_tag_version().strip()
    with open(VERSION_PATH, "w") as vf:
        vf.write(tag_version)
    return tag_version


class UploadCommand(Command):
    """
    Support setup.py upload.

    Based on, and thanks to, https://github.com/kennethreitz/setup.py/blob/master/setup.py
    """

    description = "Build and publish the package."
    user_options = []

    @staticmethod
    def status(s):
        """Prints things in bold."""
        print("\033[1m{0}\033[0m".format(s))

    def initialize_options(self):
        pass

    def finalize_options(self):
        pass

    def run(self):
        try:
            self.status("Removing previous builds…")
            rmtree(os.path.join(HERE, "dist"))
        except OSError:
            pass

        self.status("Building Source and Wheel (universal) distribution…")
        os.system("{0} setup.py sdist bdist_wheel --universal".format(sys.executable))

        self.status("Uploading the package to PyPI via Twine…")

        repo_arg = (
            "--repository-url https://test.pypi.org/legacy/"
            if os.environ.get("PYPI_ENV") == "test"
            else ""
        )

        os.system("twine upload {} dist/*".format(repo_arg))

        sys.exit()


setup(
    name="stencila-schema",
    version=get_version(),
    description="",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Stencila and contributors",
    author_email="hello@stenci.la",
    python_requires=">=3.6.0",
    url="https://github.com/stencila/schema",
    packages=["stencila.schema"],
    extras_require={},
    include_package_data=True,
    license="Apache-2.0",
    classifiers=[
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: Implementation :: CPython",
        "Programming Language :: Python :: Implementation :: PyPy",
    ],
    cmdclass={"upload": UploadCommand,},
)
