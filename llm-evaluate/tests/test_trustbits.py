import json
from pathlib import Path

import pytest

from llm_evaluate.orm import (
    LLMSnapshotRecord,
    LLMStatsRecord,
    ProviderRecord,
    close_connection,
    init_connection,
)
from llm_evaluate.provider.trustbit import TrustbitJson
from llm_evaluate.stats import build_grid, load_stats


@pytest.fixture(scope="session")
def trustbit_json(data_path) -> TrustbitJson:
    pth = data_path / "trustbit.json"
    return TrustbitJson.model_validate_json(pth.read_text())


async def test_database(with_sqlite, trustbit_json):
    # Check we can save a snapshot
    rec = trustbit_json.to_provider_record()
    await rec.save()
    pr = await ProviderRecord.first()
    assert pr.provider == "trustbit"

    # Check we can generate a snapshot
    snapshot_id = await trustbit_json.generate_snapshot()
    saved = await LLMSnapshotRecord.filter(id=snapshot_id).first()
    assert saved.provider == "trustbit"

    # Check that we can get a dataframe from the loaded snapshots
    await load_stats(snapshot_id)
