"""
Performance benchmarking of functions in the `convert` module

These benchmarks are aimed at detecting regressions in performance of
the Python-Rust bindings. They are not intended to benchmark the Rust code
(that is done elsewhere). As such, the function calls are very simple and
do little actual conversion.

Note that there is likely to be significant overhead from using
`asyncio.run` but that seems unavoidable.
"""

import asyncio
import pytest

from stencila.convert import to_string, from_string, from_path, to_path, from_to
from stencila.types import Article


def run_from_string():
    asyncio.run(from_string("""{ "type": "Article", "content": [] }"""))


@pytest.mark.skip(reason="Article.__init__ broken due to more than one base type")
@pytest.mark.benchmark(min_rounds=100)
def bench_from_string(benchmark):
    benchmark(run_from_string)


def run_to_string():
    asyncio.run(to_string(Article([])))


@pytest.mark.skip(reason="Article.__init__ broken due to more than one base type")
@pytest.mark.benchmark(min_rounds=100)
def bench_to_string(benchmark):
    benchmark(run_to_string)
