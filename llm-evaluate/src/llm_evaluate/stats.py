from itertools import product

import numpy as np
import polars as pl
from pydantic import BaseModel, field_validator

from .orm import LLMStatsRecord, LLMCategory


async def load_stats(snapshot_id: int) -> pl.DataFrame:
    """Load the stats for a snapshot"""
    records = await LLMStatsRecord.filter(snapshot_id=snapshot_id).all()
    cols = "name cost speed quality".split()
    data = [[getattr(x, c) for c in cols] for x in records]
    df = pl.DataFrame(data, schema=cols)
    return df


async def build_grid(snapshot_id: int):
    df = await load_stats(snapshot_id)
    model_names = df.select("name").to_numpy().flatten()
    arr = df.select("cost speed quality".split()).to_numpy()

    # Define the range of weights (0 to 1 in steps of 0.1)
    weights = np.arange(0, 1.1, 0.1)

    # Function to calculate the weighted score for a model
    def calculate_score(model, weight):
        return np.dot(model, weight)

    # Store the results
    results = []

    # Iterate over all possible weight combinations that add to 1.0
    for w_cost, w_speed, w_quality in product(weights, repeat=3):
        if np.isclose(
                w_cost + w_speed + w_quality, 1.0
        ):  # Ensure the sum of weights is 1
            current_weights = np.array([w_cost, w_speed, w_quality])
            scores = np.apply_along_axis(calculate_score, 1, arr, current_weights)
            best = np.argmax(scores)
            results.append(
                (w_cost, w_speed, w_quality, model_names[best], scores[best])
            )

    return pl.DataFrame(results, schema="cost speed quality model score".split())


class Result(BaseModel):
    @field_validator("cost", "speed", "quality")
    @classmethod
    def strip_non_numeric(cls, value: float) -> float:
        return round(value, 2)

    cost: float
    speed: float
    quality: float
    model: str


class CategoryResults(BaseModel):
    category: LLMCategory
    results: list[Result]

    @classmethod
    def from_grid(cls, category: LLMCategory, grid: pl.DataFrame):
        results = []
        for row in grid.to_dicts():
            results.append(Result.model_validate(row))
        return cls(category=category, results=results)


class Routing(BaseModel):
    id: int
    # provider: str
    categories: list[CategoryResults]