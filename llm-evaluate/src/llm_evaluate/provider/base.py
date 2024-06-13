from datetime import datetime

from pydantic import BaseModel

from llm_evaluate.orm import ProviderRecord


class ProviderJson(BaseModel):
    provider: str
    when: datetime

    def to_provider_record(self):
        return ProviderRecord(provider=self.provider, when=self.when, dump=self.dict())
