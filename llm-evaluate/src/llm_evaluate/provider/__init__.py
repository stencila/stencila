from .base import ProviderJson
from .trustbit import TrustbitJson
from enum import StrEnum, auto


class ProviderType(StrEnum):
    Trustbit = auto()
    # Add more providers here


PROVIDERS: dict[str, type[ProviderJson]] = {ProviderType.Trustbit: TrustbitJson}

__all__ = ["PROVIDERS", "ProviderJson", "ProviderType"]
