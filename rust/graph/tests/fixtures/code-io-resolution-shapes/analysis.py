"""Python I/O path shapes that static analysis should and should not resolve."""

from pathlib import Path
from urllib.request import Request, urlopen

# Module constant.
INPUT = "data/input.csv"

# Output written at the end of the script.
OUTPUT = "results/summary.csv"

# Literal collection iterated below.
SOURCES = ["data/first.csv", "data/second.csv"]

# Base segment used to build a template path.
BASE = "data"


def load(path):
    """One level of helper indirection."""
    with open(path) as file:
        return file.read()


def fetch(url):
    """A path-preserving wrapper bound to a local before use."""
    request = Request(url)
    with urlopen(request) as response:
        return response.read()


def main():
    # Module constant.
    text = open(INPUT).read()

    # Iteration over a literal collection.
    for source in SOURCES:
        text += open(source).read()

    # Single-assignment local.
    local = "data/local.csv"
    text += open(local).read()

    # Fully resolvable template.
    text += open(f"{BASE}/template.csv").read()

    # One level of helper function.
    text += load("data/helper.csv")

    # Inline path-preserving wrapper constructor.
    text += open(Path("data/wrapped.csv")).read()

    # Helper plus wrapper-bound remote read.
    text += str(fetch("https://example.org/remote.csv"))

    # Negative: assigned in both branches, so the value is control-flow carried.
    if text:
        conditional = "data/if-branch.csv"
    else:
        conditional = "data/else-branch.csv"
    text += open(conditional).read()

    # Negative: assigned more than once in the same scope.
    reassigned = "data/first-value.csv"
    reassigned = "data/second-value.csv"
    text += open(reassigned).read()

    # Negative: bound to an expression rather than a literal.
    computed = text[:3] + ".csv"
    text += open(computed).read()

    # Negative: template with a placeholder that cannot be resolved.
    text += open(f"{BASE}/{computed}").read()

    with open(OUTPUT, "w") as file:
        file.write(text)


main()
