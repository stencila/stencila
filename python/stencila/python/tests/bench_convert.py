"""
Performance benchmarking of functions in the `convert` module

These benchmarks are aimed at detecting regressions in performance of
the Python-Rust bindings. They are not intended to benchmark the Rust code
(that is done elsewhere). As such, the function calls are very simple and
do little actual conversion.
"""

import pytest
from stencila_types.types import Article

from stencila.convert import from_string, to_string


def run_from_string():
    from_string("""{ "type": "Article", "content": [] }""")


@pytest.mark.benchmark(min_rounds=100)
def bench_from_string(benchmark):
    benchmark(run_from_string)


def run_to_string():
    to_string(Article(content=[]))


@pytest.mark.benchmark(min_rounds=100)
def bench_to_string(benchmark):
    benchmark(run_to_string)
