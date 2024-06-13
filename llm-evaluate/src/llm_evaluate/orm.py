# Ignore Meta overrides for Tortoise models
# pyright: reportIncompatibleVariableOverride=false
#

from tortoise import Tortoise, fields
from tortoise.models import Model


async def init_connection(db_url: str):
    await Tortoise.init(db_url=db_url, modules={"models": ["llm_evaluate.orm"]})
    await Tortoise.generate_schemas()


async def close_connection():
    await Tortoise.close_connections()


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


class LLMSnapshotRecord(Model):
    id = fields.IntField(pk=True)

    # What category are we interested in?
    # TODO: This should be an Enum?
    category = fields.TextField()

    # Origin information ----
    created = fields.DatetimeField(auto_now_add=True)

    # This points to the ScrapeRecord that was used to generate this record
    provider = fields.TextField()
    when = fields.DateField()

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