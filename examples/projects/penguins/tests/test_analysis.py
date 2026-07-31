from argparse import Namespace
from pathlib import Path

import numpy as np
import pandas as pd
import pytest
import yaml

from src.analyze import (
    CANDIDATES,
    DECISIONS,
    add_size_scores,
    aicc,
    eligible,
    fit_candidates,
    isotopes,
    paper_like_outlier,
    selected_data,
)
from src.plot import dimorphism as plot_dimorphism
from src.prepare import prepare

ROOT = Path(__file__).parents[1]


@pytest.fixture(scope="module")
def normalized() -> pd.DataFrame:
    return prepare(ROOT / "penguins.csv")


def baseline_args(**overrides: str) -> Namespace:
    values = yaml.safe_load((ROOT / "universes/baseline.yaml").read_text())["decisions"]
    values.update(overrides)
    return Namespace(**values, universe_id="baseline")


def test_source_facts_and_species_island_confounding() -> None:
    source = pd.read_csv(ROOT / "penguins.csv", keep_default_na=False)
    assert source.shape == (344, 17)
    assert (source["Sex"] == "").sum() == 10
    assert (source["Sex"] == ".").sum() == 1
    assert (source["Delta 13 C (o/oo)"] == "").sum() == 13
    assert (source["Delta 15 N (o/oo)"] == "").sum() == 14
    islands = source.groupby("Species")["Island"].unique().to_dict()
    assert set(islands["Gentoo penguin (Pygoscelis papua)"]) == {"Biscoe"}
    assert set(islands["Chinstrap penguin (Pygoscelis antarctica)"]) == {"Dream"}


def test_normalization_retains_rows_and_adds_stable_quality_fields(
    normalized: pd.DataFrame,
) -> None:
    assert len(normalized) == 344
    assert normalized["row_id"].is_unique
    assert normalized["sex"].isna().sum() == 11
    assert normalized["delta_13c"].isna().sum() == 13
    assert normalized["delta_15n"].isna().sum() == 14
    assert {
        "year",
        "clutch_complete",
        "morphology_complete",
        "isotope_complete",
        "quality_issue",
    } <= set(normalized)


def test_eligibility_and_complete_case_policies(normalized: pd.DataFrame) -> None:
    strict = eligible(normalized, "complete_clutch_only")
    inclusive = eligible(normalized, "all_sampled")
    assert len(strict) < len(inclusive) == 344
    endpoint, _ = selected_data(normalized, baseline_args(), "delta_13c")
    shared, _ = selected_data(
        normalized, baseline_args(sample_policy="shared_complete_cases"), "delta_13c"
    )
    assert len(shared) <= len(endpoint)
    assert shared[["delta_13c", "delta_15n"]].notna().all(axis=None)


@pytest.mark.parametrize("metric", ["paper_pc1_unscaled", "paper_pc1_standardized"])
def test_pc1_orientation_and_scaling(normalized: pd.DataFrame, metric: str) -> None:
    scored = add_size_scores(normalized, metric)
    for _, group in scored.dropna(subset=["size_score"]).groupby("species"):
        assert np.corrcoef(group["size_score"], group["flipper_length_mm"])[0, 1] > 0
        assert group["size_score"].mean() == pytest.approx(0, abs=1e-10)
    unscaled = add_size_scores(normalized, "paper_pc1_unscaled")
    standardized = add_size_scores(normalized, "paper_pc1_standardized")
    assert not np.allclose(unscaled["size_score"].dropna(), standardized["size_score"].dropna())


def test_aicc_and_weights(normalized: pd.DataFrame) -> None:
    assert aicc(100, 50, 4) == pytest.approx(100 + 40 / 45)
    data, _ = selected_data(normalized, baseline_args(), "delta_15n")
    fits = fit_candidates(
        data.loc[data["species"].eq("Adelie")], "delta_15n", "paper_aicc_candidates"
    )
    assert {item["model"] for item in fits} == set(CANDIDATES)
    assert sum(item["weight"] for item in fits) == pytest.approx(1)
    assert min(item["delta_aicc"] for item in fits) == pytest.approx(0)


def test_outlier_is_deterministic(normalized: pd.DataFrame) -> None:
    data = add_size_scores(eligible(normalized, "complete_clutch_only"), "paper_pc1_unscaled")
    data = data.dropna(subset=["delta_13c", "sex", "year", "size_score"])
    assert paper_like_outlier(data) == paper_like_outlier(data.sample(frac=1, random_state=42))
    result = isotopes(normalized, baseline_args())
    ids = result.loc[
        result["species"].eq("Adelie") & result["isotope"].eq("delta_13c"),
        "outlier_row_id",
    ].dropna()
    assert ids.nunique() == 1


def test_universes_are_complete_and_focused() -> None:
    files = sorted((ROOT / "universes").glob("*.yaml"))
    assert len(files) == 8
    baseline = yaml.safe_load((ROOT / "universes/baseline.yaml").read_text())["decisions"]
    assert set(baseline) == set(DECISIONS)
    for path in files:
        decisions = yaml.safe_load(path.read_text())["decisions"]
        assert set(decisions) == set(DECISIONS)
        differences = sum(decisions[key] != baseline[key] for key in DECISIONS)
        assert differences == (0 if path.stem == "baseline" else 1)


def test_output_schema_and_nonempty_figure(normalized: pd.DataFrame, tmp_path: Path) -> None:
    models = isotopes(normalized, baseline_args())
    required = {"record_type", "species", "isotope", "n", "model", *DECISIONS}
    assert required <= set(models)
    rows = []
    for species in sorted(normalized["species"].unique()):
        for sex in ["Female", "Male"]:
            rows.append(
                {
                    "species": species,
                    "sex": sex,
                    "metric": "size_score",
                    "mean": 1.0,
                    "std_error": 0.1,
                }
            )
    figure = plot_dimorphism(pd.DataFrame(rows))
    output = tmp_path / "figure.png"
    figure.savefig(output)
    assert output.stat().st_size > 0
