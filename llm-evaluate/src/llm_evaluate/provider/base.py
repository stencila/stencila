from datetime import datetime
from typing import Self

from pydantic import BaseModel
from abc import ABC, abstractmethod
from ..orm import LLMCategory, RoutingRecord

from llm_evaluate.orm import ProviderRecord


class ProviderJson(BaseModel, ABC):
    provider: str
    when: datetime

    def to_provider_record(self):
        return ProviderRecord(provider=self.provider, when=self.when, dump=self.dict())

    @classmethod
    @abstractmethod
    async def scrape(cls) -> Self | None: ...

    @abstractmethod
    async def generate_snapshot(self, routing: RoutingRecord, category: LLMCategory) -> int: ...


