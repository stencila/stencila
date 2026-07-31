#!/usr/bin/env python3
"""Download the Palmer Penguins dataset to a single CSV file.

The data are downloaded from their citable DOIs on the Environmental Data
Initiative (EDI) repository, rather than from a copy on GitHub. There is one
DOI per species, each a data package published by Palmer Station Antarctica
LTER and K. Gorman. Resolving a DOI redirects to an EDI package page, whose
data entity is then fetched from the PASTA API.

The DOIs are held in a dict and fetched through a helper, which is how one
would ordinarily write this. Static analysis resolves the collection, the
helper's parameter, and the module constant, so the three datasets are still
recorded as upstream dependencies of the CSV file this writes. The entity URL
inside `download_package` genuinely does not exist until the DOI redirect has
been followed, so it is correctly left unresolved.

Gorman KB, Williams TD, Fraser WR (2014) Ecological sexual dimorphism and
environmental variability within a community of Antarctic penguins (genus
Pygoscelis). PLoS ONE 9(3):e90081. https://doi.org/10.1371/journal.pone.0090081
"""

import argparse
import csv
import io
import re
from urllib.request import Request, urlopen

PASTA = "https://pasta.lternet.edu/package"
USER_AGENT = "PalmerPenguinsASTRA/1.0 (reproducible research download)"

DOIS = {
    "Adelie": "https://doi.org/10.6073/pasta/abc50eed9138b75f54eaada0841b9b86",
    "Gentoo": "https://doi.org/10.6073/pasta/2b1cff60f81640f182433d23e68541ce",
    "Chinstrap": "https://doi.org/10.6073/pasta/409c808f8fc9899d02401bdb04580af7",
}


def package_id(url: str) -> str:
    """Extract the EDI package id (e.g. `knb-lter-pal.219.3`) from a package URL."""
    match = re.search(r"packageid=([\w.-]+)", url)
    if not match:
        raise ValueError(f"Unable to find a package id in URL: {url}")
    return match.group(1)


def fetch(url: str, accept: str | None = None) -> str:
    """Fetch a URL as text, optionally negotiating a content type."""
    headers = {"User-Agent": USER_AGENT}
    if accept:
        headers["Accept"] = accept
    request = Request(url, headers=headers)
    with urlopen(request) as response:
        return response.read().decode()


def resolve() -> list[str]:
    """Resolve the DOI of each species to the EDI package id it identifies."""
    identifiers = []
    for doi in DOIS.values():
        with urlopen(Request(doi, headers={"User-Agent": USER_AGENT})) as response:
            identifiers.append(package_id(response.url))
    return identifiers


def download_package(identifier: str) -> str:
    """Download the data entity of an EDI package as CSV text."""
    scope, number, revision = identifier.split(".")
    base = f"{PASTA}/data/eml/{scope}/{number}/{revision}"
    entity_id = fetch(base).split()[0]
    return fetch(f"{base}/{entity_id}")


def download(output: str) -> None:
    """Download the data for all three species into a single CSV file."""
    with open(output, "w", newline="") as file:
        writer = csv.writer(file)
        header = None
        for package in resolve():
            rows = csv.reader(io.StringIO(download_package(package)))
            columns = next(rows)
            if header is None:
                header = columns
                writer.writerow(header)
            elif columns != header:
                raise ValueError(f"Unexpected columns in package {package}: {columns}")
            writer.writerows(rows)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default="penguins.csv")
    args = parser.parse_args()
    download(args.output)
    print(f"Wrote {args.output}")
