# Ignore Meta overrides for Tortoise models
# pyright: reportIncompatibleVariableOverride=false
#
from enum import StrEnum, auto

from tortoise import Tortoise, fields
from tortoise.models import Model

from llm_evaluate.settings import get_settings


async def init_connection(db_url: str):
    await Tortoise.init(db_url=db_url, modules={"models": ["llm_evaluate.orm"]})
    await Tortoise.generate_schemas()


async def close_connection():
    await Tortoise.close_connections()


class Database:
    def __init__(self):
        self.url = f"sqlite:///{get_settings().database_path}"

    async def __aenter__(self):
        await init_connection(self.url)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await close_connection()


class LLMCategory(StrEnum):
    Code = auto()
    Text = auto()


# ORM models below -----------
class ProviderRecord(Model):
    """Record with JSON dump from a provider"""

    id = fields.IntField(pk=True)
    provider = fields.TextField()
    when = fields.DateField()
    # created = fields.DatetimeField(auto_now_add=True)
    dump = fields.JSONField()

    class Meta:
        table = "provider"
        # We should only get one record per provider per day
        unique_together = (("provider", "when"),)


class RoutingRecord(Model):
    """Record with JSON dump from a provider"""

    id = fields.IntField(pk=True)
    created = fields.DatetimeField(auto_now_add=True)

    # This points to the ScrapeRecord that was used to generate this record
    provider: fields.ForeignKeyRelation[ProviderRecord] = fields.ForeignKeyField(
        "models.ProviderRecord", related_name="routing"
    )

    categories: fields.ReverseRelation["LLMSnapshotRecord"]

    class Meta:
        table = "routing"


class LLMSnapshotRecord(Model):
    id = fields.IntField(pk=True)

    # What category are we interested in?
    category = fields.CharEnumField(LLMCategory)

    routing: fields.ForeignKeyRelation[RoutingRecord] = fields.ForeignKeyField(
        "models.RoutingRecord", related_name="categories"
    )
    stats: fields.ReverseRelation["LLMStatsRecord"]

    class Meta:
        table = "llm_snapshot"


class LLMStatsRecord(Model):
    """Records of LLM performance in Stencila format"""

    id = fields.IntField(pk=True)

    # Stencila name for Model
    name = fields.TextField()

    # Normalized Metrics: Quality / Cost / Speed
    quality = fields.FloatField()
    cost = fields.FloatField()
    speed = fields.FloatField()

    snapshot: fields.ForeignKeyRelation[LLMSnapshotRecord] = fields.ForeignKeyField(
        "models.LLMSnapshotRecord", related_name="stats"
    )

    class Meta:
        table = "llm_stats"
