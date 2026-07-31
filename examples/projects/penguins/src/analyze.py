#!/usr/bin/env python3
"""Audit, summarize, and model normalized Palmer Penguins observations."""

import argparse
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import pandas as pd
import yaml

MORPHOLOGY = ["culmen_length_mm", "culmen_depth_mm", "flipper_length_mm"]
ISOTOPES = ["delta_13c", "delta_15n"]
DECISIONS = [
    "eligibility",
    "sample_policy",
    "size_metric",
    "model_strategy",
    "outlier_policy",
    "uncertainty_method",
]
CANDIDATES = {
    "null": "1",
    "sex": "C(sex, Treatment(reference='Female'))",
    "size": "size_score",
    "year": "C(year, Treatment(reference=2007))",
    "sex_year": "C(sex, Treatment(reference='Female')) + C(year, Treatment(reference=2007))",
    "size_year": "size_score + C(year, Treatment(reference=2007))",
    "sex_x_year": "C(sex, Treatment(reference='Female')) * C(year, Treatment(reference=2007))",
    "size_x_year": "size_score * C(year, Treatment(reference=2007))",
}
JOINT = {
    "joint_sex_size": (
        "C(sex, Treatment(reference='Female')) + size_score + C(year, Treatment(reference=2007))"
    )
}


def load(path: Path) -> pd.DataFrame:
    frame = pd.read_csv(path)
    frame["year"] = pd.to_numeric(frame["year"], errors="coerce")
    return frame


def eligible(frame: pd.DataFrame, policy: str) -> pd.DataFrame:
    if policy == "complete_clutch_only":
        return frame.loc[frame["clutch_complete"].astype(str).str.lower().eq("true")].copy()
    if policy == "all_sampled":
        return frame.copy()
    raise ValueError(f"Unknown eligibility policy: {policy}")


def add_size_scores(frame: pd.DataFrame, metric: str) -> pd.DataFrame:
    """Add species-specific, positively oriented structural-size scores."""
    result = frame.copy()
    result["size_score"] = np.nan
    if metric == "body_mass":
        result["size_score"] = result["body_mass_g"]
        return result
    if metric not in {"paper_pc1_unscaled", "paper_pc1_standardized"}:
        raise ValueError(f"Unknown size metric: {metric}")

    for _species, group in result.groupby("species", sort=True):
        valid = group[MORPHOLOGY].dropna()
        values = valid.to_numpy(dtype=float).copy()
        values -= values.mean(axis=0)
        if metric == "paper_pc1_standardized":
            values /= values.std(axis=0, ddof=0)
        _, _, loadings = np.linalg.svd(values, full_matrices=False)
        scores = values @ loadings[0]
        if np.corrcoef(scores, valid["flipper_length_mm"])[0, 1] < 0:
            scores *= -1
        result.loc[valid.index, "size_score"] = scores
    return result


def endpoint_sample(frame: pd.DataFrame, endpoint: str, policy: str) -> pd.DataFrame:
    required = [endpoint, "sex", "year", "size_score"]
    if policy == "shared_complete_cases":
        required.extend(ISOTOPES)
    elif policy != "endpoint_complete_cases":
        raise ValueError(f"Unknown sample policy: {policy}")
    return frame.dropna(subset=list(dict.fromkeys(required))).copy()


def aicc(aic: float, nobs: int, parameters: int) -> float:
    denominator = nobs - parameters - 1
    return np.inf if denominator <= 0 else aic + (2 * parameters * (parameters + 1)) / denominator


@dataclass
class Fit:
    params: pd.Series
    bse: pd.Series
    aic: float
    nobs: int
    df_model: int
    rsquared: float
    external_residuals: np.ndarray


def design_matrix(data: pd.DataFrame, model: str) -> tuple[np.ndarray, list[str]]:
    """Construct the explicitly declared candidate design matrix."""
    columns = [np.ones(len(data))]
    names = ["Intercept"]
    uses_sex = "sex" in model
    uses_size = "size" in model
    uses_year = "year" in model or model == "joint_sex_size"
    interactions = "_x_year" in model
    if uses_sex:
        columns.append(data["sex"].eq("Male").to_numpy(dtype=float))
        names.append("sex[Male]")
    if uses_size:
        columns.append(data["size_score"].to_numpy(dtype=float))
        names.append("size_score")
    year_columns = []
    if uses_year:
        for year in sorted(data["year"].dropna().unique()):
            if int(year) == 2007:
                continue
            values = data["year"].eq(year).to_numpy(dtype=float)
            year_columns.append((int(year), values))
            columns.append(values)
            names.append(f"year[{int(year)}]")
    if interactions:
        focal = columns[1]
        focal_name = names[1]
        for year, values in year_columns:
            columns.append(focal * values)
            names.append(f"{focal_name}:year[{year}]")
    return np.column_stack(columns), names


def fit_ols(data: pd.DataFrame, endpoint: str, model: str) -> Fit:
    x, names = design_matrix(data, model)
    y = data[endpoint].to_numpy(dtype=float)
    inverse = np.linalg.pinv(x.T @ x)
    coefficients = inverse @ x.T @ y
    residuals = y - x @ coefficients
    nobs, parameters = x.shape
    rss = float(residuals @ residuals)
    sigma2 = rss / (nobs - parameters)
    bse = np.sqrt(np.diag(inverse) * sigma2)
    centered = y - y.mean()
    total = float(centered @ centered)
    leverage = np.sum((x @ inverse) * x, axis=1)
    deleted_sse = rss - residuals**2 / np.clip(1 - leverage, 1e-12, None)
    deleted_mse = deleted_sse / (nobs - parameters - 1)
    external = residuals / np.sqrt(deleted_mse * np.clip(1 - leverage, 1e-12, None))
    log_likelihood_component = nobs * (np.log(2 * np.pi) + 1 + np.log(rss / nobs))
    return Fit(
        params=pd.Series(coefficients, index=names),
        bse=pd.Series(bse, index=names),
        aic=float(log_likelihood_component + 2 * parameters),
        nobs=nobs,
        df_model=parameters - 1,
        rsquared=1 - rss / total,
        external_residuals=external,
    )


def fit_candidates(data: pd.DataFrame, endpoint: str, strategy: str) -> list[dict]:
    candidates = CANDIDATES if strategy == "paper_aicc_candidates" else JOINT
    fitted: list[dict] = []
    for name in candidates:
        model = fit_ols(data, endpoint, name)
        fitted.append(
            {
                "model": name,
                "fit": model,
                "aicc": aicc(model.aic, int(model.nobs), int(model.df_model) + 1),
            }
        )
    minimum = min(item["aicc"] for item in fitted)
    relative = np.array([np.exp(-0.5 * (item["aicc"] - minimum)) for item in fitted])
    weights = relative / relative.sum()
    for item, weight in zip(fitted, weights, strict=True):
        item["delta_aicc"] = item["aicc"] - minimum
        item["weight"] = weight
    return fitted


def model_average(fitted: list[dict]) -> list[dict]:
    """Full-average coefficients and unconditional uncertainty over candidate models."""
    terms = sorted(set().union(*(set(item["fit"].params.index) for item in fitted)))
    rows = []
    for term in terms:
        estimates = np.array([item["fit"].params.get(term, 0.0) for item in fitted])
        variances = np.array([item["fit"].bse.get(term, 0.0) ** 2 for item in fitted])
        weights = np.array([item["weight"] for item in fitted])
        estimate = float(np.sum(weights * estimates))
        variance = float(np.sum(weights * (variances + (estimates - estimate) ** 2)))
        se = np.sqrt(variance)
        rows.append(
            {
                "term": term,
                "estimate": estimate,
                "std_error": se,
                "conf_low": estimate - 1.96 * se,
                "conf_high": estimate + 1.96 * se,
            }
        )
    return rows


def paper_like_outlier(data: pd.DataFrame) -> str | None:
    """Select the single Adelie carbon outlier from maximal interaction residuals."""
    adelie = data.loc[data["species"].eq("Adelie")].copy()
    if adelie.empty:
        return None
    scores = pd.Series(0.0, index=adelie.index)
    for candidate in ("sex_x_year", "size_x_year"):
        model = fit_ols(adelie, "delta_13c", candidate)
        residuals = pd.Series(np.abs(model.external_residuals), index=adelie.index)
        scores = pd.concat([scores, residuals], axis=1).max(axis=1)
    ranked = adelie.assign(_score=scores).sort_values(
        ["_score", "row_id"], ascending=[False, True], kind="stable"
    )
    return str(ranked.iloc[0]["row_id"])


def selected_data(
    frame: pd.DataFrame, args: argparse.Namespace, endpoint: str
) -> tuple[pd.DataFrame, str | None]:
    data = add_size_scores(eligible(frame, args.eligibility), args.size_metric)
    data = endpoint_sample(data, endpoint, args.sample_policy)
    outlier_id = None
    if args.outlier_policy == "paper_like_single_exclusion" and endpoint == "delta_13c":
        outlier_id = paper_like_outlier(data)
        if outlier_id is not None:
            data = data.loc[~data["row_id"].eq(outlier_id)].copy()
    elif args.outlier_policy not in {"paper_like_single_exclusion", "retain_all"}:
        raise ValueError(f"Unknown outlier policy: {args.outlier_policy}")
    return data, outlier_id


def audit(frame: pd.DataFrame, args: argparse.Namespace) -> pd.DataFrame:
    base = add_size_scores(eligible(frame, args.eligibility), args.size_metric)
    rows = []
    for species, group in base.groupby("species", sort=True):
        carbon, outlier = selected_data(frame, args, "delta_13c")
        nitrogen, _ = selected_data(frame, args, "delta_15n")
        rows.append(
            {
                "species": species,
                "source_n": int(frame["species"].eq(species).sum()),
                "eligible_n": len(group),
                "sex_missing_n": int(group["sex"].isna().sum()),
                "morphology_missing_n": int(group[MORPHOLOGY].isna().any(axis=1).sum()),
                "delta_13c_missing_n": int(group["delta_13c"].isna().sum()),
                "delta_15n_missing_n": int(group["delta_15n"].isna().sum()),
                "delta_13c_model_n": int(carbon["species"].eq(species).sum()),
                "delta_15n_model_n": int(nitrogen["species"].eq(species).sum()),
                "excluded_outlier_row_id": outlier if species == "Adelie" else None,
            }
        )
    return with_decisions(pd.DataFrame(rows), args)


def dimorphism(frame: pd.DataFrame, args: argparse.Namespace) -> pd.DataFrame:
    data = add_size_scores(eligible(frame, args.eligibility), args.size_metric)
    required = ["sex", "year", "size_score"]
    if args.sample_policy == "shared_complete_cases":
        required.extend(ISOTOPES)
    data = data.dropna(subset=required)
    rows = []
    for (species, sex), group in data.groupby(["species", "sex"], sort=True):
        for metric in [*MORPHOLOGY, "body_mass_g", "size_score"]:
            values = group[metric].dropna()
            rows.append(
                {
                    "species": species,
                    "sex": sex,
                    "metric": metric,
                    "n": len(values),
                    "mean": values.mean(),
                    "std_error": values.std(ddof=1) / np.sqrt(len(values)),
                }
            )
    result = pd.DataFrame(rows)
    means = result.pivot_table(index=["species", "metric"], columns="sex", values="mean")
    means["dimorphism_percent"] = 100 * (means["Male"] / means["Female"] - 1)
    result = result.merge(means["dimorphism_percent"], on=["species", "metric"], how="left")
    # A percentage ratio is undefined for a centered score whose zero is
    # arbitrary; retain it only for measurements on ratio scales.
    result.loc[result["metric"].eq("size_score"), "dimorphism_percent"] = np.nan
    return with_decisions(result, args)


def bootstrap_joint(data: pd.DataFrame, endpoint: str, iterations: int = 2000) -> list[dict]:
    """Stratified bootstrap for joint-model sex and size coefficients."""
    design = pd.get_dummies(
        data[["sex", "year"]], columns=["sex", "year"], drop_first=True, dtype=float
    )
    design.insert(0, "intercept", 1.0)
    design["size_score"] = data["size_score"].to_numpy()
    columns = [column for column in design if column.startswith("sex_")] + ["size_score"]
    x = design.to_numpy(dtype=float)
    y = data[endpoint].to_numpy(dtype=float)
    positions = pd.Series(np.arange(len(data)), index=data.index)
    strata = [
        positions.loc[index].to_numpy() for _, index in data.groupby(["year", "sex"]).groups.items()
    ]
    rng = np.random.default_rng(20250308)
    estimates = {column: [] for column in columns}
    for _ in range(iterations):
        sample = np.concatenate(
            [rng.choice(index, size=len(index), replace=True) for index in strata]
        )
        coefficients = np.linalg.lstsq(x[sample], y[sample], rcond=None)[0]
        for column in columns:
            estimates[column].append(coefficients[design.columns.get_loc(column)])
    return [
        {
            "term": column,
            "estimate": float(np.mean(values)),
            "std_error": float(np.std(values, ddof=1)),
            "conf_low": float(np.quantile(values, 0.025)),
            "conf_high": float(np.quantile(values, 0.975)),
            "bootstrap_iterations": iterations,
        }
        for column, values in estimates.items()
    ]


def isotopes(frame: pd.DataFrame, args: argparse.Namespace) -> pd.DataFrame:
    rows = []
    for endpoint in ISOTOPES:
        selected, outlier_id = selected_data(frame, args, endpoint)
        for species, data in selected.groupby("species", sort=True):
            fitted = fit_candidates(data, endpoint, args.model_strategy)
            for item in fitted:
                reported_outlier = outlier_id if species == "Adelie" else None
                rows.append(
                    {
                        "record_type": "candidate",
                        "species": species,
                        "isotope": endpoint,
                        "n": int(item["fit"].nobs),
                        "model": item["model"],
                        "aicc": item["aicc"],
                        "delta_aicc": item["delta_aicc"],
                        "weight": item["weight"],
                        "r_squared": item["fit"].rsquared,
                        "outlier_row_id": reported_outlier,
                    }
                )
            for averaged in model_average(fitted):
                rows.append(
                    {
                        "record_type": "model_average",
                        "species": species,
                        "isotope": endpoint,
                        "n": len(data),
                        "model": "full_average",
                        "outlier_row_id": reported_outlier,
                        **averaged,
                    }
                )
            if args.uncertainty_method == "stratified_bootstrap":
                for estimate in bootstrap_joint(data, endpoint):
                    rows.append(
                        {
                            "record_type": "bootstrap",
                            "species": species,
                            "isotope": endpoint,
                            "n": len(data),
                            "model": "joint_sex_size",
                            "outlier_row_id": reported_outlier,
                            **estimate,
                        }
                    )
            elif args.uncertainty_method != "analytic":
                raise ValueError(f"Unknown uncertainty method: {args.uncertainty_method}")
    return with_decisions(pd.DataFrame(rows), args)


def with_decisions(frame: pd.DataFrame, args: argparse.Namespace) -> pd.DataFrame:
    for decision in DECISIONS:
        frame[decision] = getattr(args, decision)
    if getattr(args, "universe_id", None):
        frame["universe"] = args.universe_id
    return frame


def sensitivity(paths: Iterable[Path], configs: Iterable[Path]) -> pd.DataFrame:
    config_by_id = {}
    for path in configs:
        config = yaml.safe_load(path.read_text())
        config_by_id[config["id"]] = config["decisions"]
    rows = []
    for path in paths:
        frame = pd.read_csv(path)
        candidates = frame.loc[frame["record_type"].eq("candidate")].copy()
        universe = str(frame["universe"].dropna().iloc[0])
        for (species, isotope), group in candidates.groupby(["species", "isotope"], sort=True):
            top = group.sort_values(["aicc", "model"], kind="stable").iloc[0]
            sex_weight = group.loc[group["model"].str.contains("sex"), "weight"].sum()
            size_weight = group.loc[group["model"].str.contains("size"), "weight"].sum()
            row = {
                "universe": universe,
                "species": species,
                "isotope": isotope,
                "n": int(top["n"]),
                "top_model": top["model"],
                "top_weight": top["weight"],
                "sex_evidence_weight": sex_weight,
                "size_evidence_weight": size_weight,
                "sex_minus_size_weight": sex_weight - size_weight,
                "outlier_row_id": top.get("outlier_row_id"),
            }
            row.update(config_by_id[universe])
            rows.append(row)
    return pd.DataFrame(rows).sort_values(["universe", "species", "isotope"])


def parser() -> argparse.ArgumentParser:
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("mode", choices=["audit", "dimorphism", "isotopes", "sensitivity"])
    cli.add_argument("--input", required=True, type=Path, nargs="+")
    cli.add_argument("--output", required=True, type=Path)
    cli.add_argument("--universe-config", type=Path, nargs="*", default=[])
    cli.add_argument("--universe-id")
    cli.add_argument(
        "--eligibility",
        choices=["complete_clutch_only", "all_sampled"],
        default="complete_clutch_only",
    )
    cli.add_argument(
        "--sample-policy",
        choices=["endpoint_complete_cases", "shared_complete_cases"],
        default="endpoint_complete_cases",
    )
    cli.add_argument(
        "--size-metric",
        choices=["paper_pc1_unscaled", "paper_pc1_standardized", "body_mass"],
        default="paper_pc1_unscaled",
    )
    cli.add_argument(
        "--model-strategy",
        choices=["paper_aicc_candidates", "joint_sex_size"],
        default="paper_aicc_candidates",
    )
    cli.add_argument(
        "--outlier-policy",
        choices=["paper_like_single_exclusion", "retain_all"],
        default="paper_like_single_exclusion",
    )
    cli.add_argument(
        "--uncertainty-method", choices=["analytic", "stratified_bootstrap"], default="analytic"
    )
    return cli


def main() -> None:
    args = parser().parse_args()
    if args.mode == "sensitivity":
        result = sensitivity(args.input, args.universe_config)
    else:
        frame = load(args.input[0])
        result = {"audit": audit, "dimorphism": dimorphism, "isotopes": isotopes}[args.mode](
            frame, args
        )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    result.to_csv(args.output, index=False)
    print(f"Wrote {len(result)} rows to {args.output}")


if __name__ == "__main__":
    main()
