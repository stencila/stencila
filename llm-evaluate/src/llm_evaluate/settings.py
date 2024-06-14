from functools import cache
from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env", env_file_encoding="utf-8", env_prefix="lemmy_"
    )

    # TODO: Should be a DSN?
    database_path: Path


@cache
def get_settings() -> Settings:
    return Settings()
