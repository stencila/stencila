"""Scraping code for Trustbit"""

import re
from datetime import datetime
from typing import Self

import aiohttp
from bs4 import BeautifulSoup, element
from pydantic import BaseModel, field_validator

from llm_evaluate.orm import (
    LLMCategory,
    LLMSnapshotRecord,
    LLMStatsRecord,
    RoutingRecord,
)
from llm_evaluate.util import find_stencila_model, normalize_numbers

from .base import ProviderJson

# URL for the Trustbit leaderboard
TRUSTBIT_URL = "https://www.trustbit.tech/en/llm-leaderboard-{month}-{year}/"

# Trustbit uses German month names in the URL
GERMAN_MONTHS = [
    "januar",
    "februar",
    "marz",
    "april",
    "mai",
    "juni",
    "juli",
    "august",
    "september",
    "oktober",
    "november",
    "dezember",
]

# Trustbit requires a user-agent in the header
USER_AGENT_STRING = " ".join(
    [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
        "AppleWebKit/537.36 (KHTML, like Gecko)",
        "Chrome/91.0.4472.124 Safari/537.36",
    ]
)
HEADERS = {"User-Agent": USER_AGENT_STRING}
STATUS_OK = 200


# This is what we should find in the table.
EXPECTED_HEADERS = [
    "model",
    "code",
    "crm",
    "docs",
    "integrate",
    "marketing",
    "reason",
    "final",
    "cost",
    "speed",
]


def strip_non_ascii(value: str) -> str:
    value = re.sub(r"[^\x00-\x7F]+", "", value)
    value = value.strip().lower()
    return value


async def check_url_exists(url: str):
    async with aiohttp.ClientSession(headers=HEADERS) as session:
        try:
            async with session.head(url, allow_redirects=True) as response:
                return response.status == STATUS_OK
        except aiohttp.ClientError:
            return False


async def fetch(url: str):
    # Trustbit requires a user-agent in the header
    async with (
        aiohttp.ClientSession(headers=HEADERS) as session,
        session.get(url) as response,
    ):
        if response.status != STATUS_OK:
            raise RuntimeError(f"Failed to fetch URL: {url}, got {response.status}")
        return await response.text()


async def scrape_table(url: str):
    html = await fetch(url)
    soup = BeautifulSoup(html, "html.parser")

    table = soup.find("table")
    if not isinstance(table, element.Tag):
        raise RuntimeError("No table found on page")

    headers = [strip_non_ascii(header.text) for header in table.find_all("th")]
    if headers != EXPECTED_HEADERS:
        raise RuntimeError("Unexpected headers in table (website may have changed)")

    rows: list[list[str]] = []
    for row in table.find_all("tr")[1:]:  # Skipping the header row
        cells = [cell.text.strip() for cell in row.find_all("td")]
        rows.append(cells)

    return headers, rows


class TrustbitResult(BaseModel):
    """A record of the performance of a model from Trustbit"""

    @field_validator("name", mode="before")
    @classmethod
    def strip_non_ascii(cls, value: str) -> str:
        return strip_non_ascii(value)

    name: str

    code: int
    crm: int
    docs: int
    integrate: int
    marketing: int
    reason: int
    final: int

    @field_validator("cost_euros", "speed_tpm", mode="before")
    @classmethod
    def strip_non_numeric(cls, value: str) -> str:
        # Use regex to keep only digits and period
        if isinstance(value, str):
            value = re.sub(r"[^0-9.]", "", value)
        return value

    cost_euros: float
    speed_tpm: float


class TrustbitJson(ProviderJson):
    """Performance set from Trustbit"""

    results: list[TrustbitResult]

    @classmethod
    def _build(cls, when: datetime, headers: list[str], rows: list[list[str]]) -> Self:
        results: list[TrustbitResult] = []
        # Pydantic aliases are way too confusing.
        rename = {"cost": "cost_euros", "speed": "speed_tpm", "model": "name"}

        for row in rows:
            dct = dict(zip(headers, row, strict=True))
            for key, value in rename.items():
                dct[value] = dct.pop(key)
            model = TrustbitResult.model_validate(dct)
            results.append(model)

        return cls(provider="trustbit", when=when, results=results)

    @classmethod
    async def scrape(cls) -> Self | None:
        now = datetime.now()
        current_month = now.month - 1
        current_year = now.year

        # TODO: Fix when so that it is the exact date
        while current_month >= 0:
            url = TRUSTBIT_URL.format(
                month=GERMAN_MONTHS[current_month], year=current_year
            )
            if await check_url_exists(url):
                headers, rows = await scrape_table(url)
                return TrustbitJson._build(when=now, headers=headers, rows=rows)

            # TODO: Logging
            # print("Winding back")
            current_month -= 1

        return None

    async def generate_snapshot(
        self, routing: RoutingRecord, category: LLMCategory
    ) -> int:
        """Generate a *normalized* snapshot of these results"""
        snapshot = await LLMSnapshotRecord.create(
            routing=routing, category=category, provider="trustbit", when=self.when
        )
        costs = normalize_numbers([result.cost_euros for result in self.results])
        speeds = normalize_numbers([result.speed_tpm for result in self.results])
        if category == LLMCategory.Code:
            qualities = normalize_numbers([result.code for result in self.results])
        elif category == LLMCategory.Text:
            # Not sure if this is the best?
            qualities = normalize_numbers([result.docs for result in self.results])
        else:
            raise ValueError(f"Unknown category: {category}")

        for result, cost, speed, quality in zip(
            self.results, costs, speeds, qualities, strict=True
        ):
            # TODO: This is pretty broken for now
            name = find_stencila_model(result.name, cutoff=7)
            if name is None:
                continue
            print(f"matching {result.name} to {name}")
            await LLMStatsRecord.create(
                snapshot=snapshot,
                name=name,
                quality=quality,
                # Use complement, as we'll seek a max
                cost=1.0 - cost,
                speed=speed,
            )

        return snapshot.id


if __name__ == "__main__":
    import asyncio

    dump = asyncio.run(TrustbitJson.scrape())
    with open("trustbit.json", "w") as fp:
        json_string = dump.model_dump_json(indent=4)
        fp.write(json_string)

    # print(dump.json())
