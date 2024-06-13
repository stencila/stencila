from typing import Protocol, Self

from .trustbit import TrustbitJson


class Provider(Protocol):
    @classmethod
    async def scrape(cls) -> Self | None: ...
    async def generate_snapshot(self, category: str = "*") -> int: ...


PROVIDERS: dict[str, type[Provider]] = {"trustbit": TrustbitJson}

__all__ = ["PROVIDERS", "Provider"]
