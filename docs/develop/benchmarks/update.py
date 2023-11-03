#!/usr/bin/env python

"""
This script is for parsing, normalization and collating benchmark results
for each commit.

At present it does not do any presentation of benchmarks but the intention
is to dog-food that by making this a Stencila document. For now,
presentation is done in the sibling `index.html` file.
"""

import json
import re
import os
from datetime import datetime
import subprocess
import sys

REPO_ROOT = "../../.."


def shell(command):
    """Run a shell command and get stdout"""
    return (
        subprocess.run(command, shell=True, stdout=subprocess.PIPE)
        .stdout.decode("utf-8")
        .strip()
    )


def divan(content):
    """
    Parse results from running benchmarks using the Rust `divan` crate
    """

    header_re = re.compile(r"^(\w+)\s+fastest")
    times_re = re.compile(
        #           name    fastest                slowest                median                 mean                   samples     iters
        r"^(?:├|╰)─ (\w+)\s+([\d,.]+) (µ|ms)\s+│\s+([\d,.]+) (µ|ms)\s+│\s+([\d,.]+) (µ|ms)\s+│\s+([\d,.]+) (µ|ms)\s+│\s+(\d+)\s+│\s+(\d+)"
    )

    prefix = ""
    results = []
    for line in content.splitlines():
        if len(line) == 0:
            continue

        header = header_re.match(line)
        if header:
            prefix = header.group(1)

        times = times_re.match(line)
        if times:
            (
                name,
                fastest,
                fastest_unit,
                slowest,
                slowest_unit,
                median,
                median_unit,
                mean,
                mean_unit,
                samples,
                iters,
            ) = times.groups()

            median = float(median)
            if median_unit == "µs":
                median /= 1e6
            else:
                median /= 1e3

            fastest = float(fastest)
            if fastest_unit == "µs":
                fastest /= 1e6
            else:
                fastest /= 1e3

            slowest = float(slowest)
            if slowest_unit == "µs":
                slowest /= 1e6
            else:
                slowest /= 1e3

            ops = int(iters)

            results.append(
                dict(
                    name=f"{prefix}.{name}",
                    unit="ops/sec",
                    value=round(ops / median, 1),
                    min=round(ops / slowest, 1),
                    max=round(ops / fastest, 1),
                    samples=int(samples),
                )
            )

    return results


def benchmarkjs(content):
    """
    Parse results from running benchmarks using the `benchmark.js`
    """

    times_re = re.compile(
        #  name    mean                 range               samples
        r"^([\w.]+) x ([\d,.]+) ops/sec ((?:±|\+-)[^%]+%) \((\d+) runs sampled\)"
    )

    results = []
    for line in content.splitlines():
        if len(line) == 0:
            continue

        times = times_re.match(line)
        if times:
            (name, mean, range, samples) = times.groups()

            results.append(
                dict(
                    name=name,
                    unit="ops/sec",
                    value=float(mean.replace(",", "")),
                    samples=int(samples),
                    other=f"range: {range}",
                )
            )

    return results


def pytest(content):
    """
    Parse results from running benchmarks using `pytest`
    """
    results = json.loads(content)
    return [
        dict(
            name=bench["name"],
            unit="ops/sec",
            value=bench["stats"]["ops"],
            samples=bench["stats"]["rounds"],
        )
        for bench in results["benchmarks"]
    ]


# The benchmarks included in each record
benchmarks = {
    "codecs": ("rust/codecs/benches/results.txt", divan),
    "node": ("node/bench/results.txt", benchmarkjs),
    "python": ("python/benchmarks.json", pytest),
}

# Read the existing records
with open("data.json") as file:
    records = json.load(file)

if "--re-parse" in sys.argv:
    # Re-parse each record to allow for fixes to parser functions
    for record in records:
        for bench in record["benches"]:
            if bench["name"] in benchmarks:
                (path, parser) = benchmarks[bench["name"]]
                bench["results"] = parser(bench["content"])

if "--no-record" not in sys.argv:
    # Read each benchmark result file, parse it, and add to the current set of benches
    benches = []
    for name, (path, parser) in benchmarks.items():
        try:
            with open(os.path.join(REPO_ROOT, path)) as file:
                content = file.read()
        except IOError:
            continue

        results = parser(content)
        benches.append(dict(name=name, file=path, content=content, results=results))

    # Add a record if there are any benches
    if len(benches) > 0:
        record = dict(
            datetime=datetime.utcnow().isoformat(),
            commit=dict(
                hash=shell("git rev-parse HEAD"),
                author=shell("git --no-pager show -s --format='%an <%ae>' HEAD"),
                committer=shell("git --no-pager show -s --format='%cn <%ce>' HEAD"),
                timestamp=shell("git --no-pager show -s --format='%ct' HEAD"),
                message=shell("git --no-pager show -s --format='%B' HEAD"),
            ),
            benches=benches,
        )
        records.append(record)

# Write records back to disk
with open("data.json", "w") as file:
    json.dump(records, file, indent=2)
