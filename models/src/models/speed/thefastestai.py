"""
Module for downloading and processing model speed metrics based on
data collected by https://thefastest.ai.
"""

from datetime import datetime, timedelta
import json
from os import path, makedirs

import requests

REGIONS = ["cdg", "iad", "sea"]

# Local directory with downloaded files
downloads = path.join(path.dirname(path.abspath(__file__)), "downloads", "thefastestai")
makedirs(downloads, exist_ok=True)


def download_date(date: datetime):
    """
    Download files for a date (if not already present)
    """
    formatted_date = date.strftime("%Y-%m-%d")
    for region in REGIONS:
        file_name = f"{region}-{formatted_date}.json"
        file_path = path.join(downloads, file_name)
        if path.exists(file_path):
            continue

        url = f"https://storage.googleapis.com/thefastest-data/{region}/text/{formatted_date}.json"
        response = requests.get(url, stream=True)

        if response.status_code == 404:
            continue

        response.raise_for_status()

        print(f"Downloading {file_name}")
        with open(file_path, "wb") as file:
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    file.write(chunk)


def download_all():
    """
    Download data for all dates since start of dataset to the current date
    """
    start_date = datetime(2024, 4, 13)
    end_date = datetime.now()

    date = start_date
    while date <= end_date:
        download_date(date)
        date += timedelta(days=1)


def score_date(date: datetime) -> dict:
    """
    Create a normalized score for each model for a date

    For each model, the mean of `total_time` is calculated across all regions.
    The mean is then scaled to have a minimum of zero and a maximum of one.
    """
    formatted_date = date.strftime("%Y-%m-%d")

    # Get total times across regions
    times = {}
    for region in REGIONS:
        file_name = f"{region}-{formatted_date}.json"
        file_path = path.join(downloads, file_name)
        if not path.exists(file_path):
            continue
        with open(file_path, "r") as file:
            data = json.load(file)

        for result in data["results"]:
            model = result["model"]
            time = result["total_time"]
            if time:
                if model in times:
                    times[model].append(time)
                else:
                    times[model] = [time]

    # Calculate the speed (inverse of mean time) by model, and min and max
    # across all models
    scores = {}
    min = 1e6
    max = 0
    for model, times in times.items():
        if times:
            speed = 1.0 / (sum(times) / float(len(times)))
            scores[model] = speed
            if speed < min:
                min = speed
            if speed > max:
                max = speed

    # Normalize scores by min and max
    range = max - min
    for model, score in scores.items():
        scores[model] = round((score - min) / range, 6)

    return scores


def score_all():
    """
    Score models for all dates since start of dataset to the current date
    """
    start_date = datetime(2024, 4, 13)
    end_date = datetime.now()

    rows = []

    # Iterate over all dates and append to rows
    date = start_date
    while date <= end_date:
        scores = score_date(date)

        date_str = date.strftime("%Y-%m-%d")
        for model, score in scores.items():
            (provider, location, model) = parse_model(model)
            rows.append((date_str, provider, location, model, score))

        date += timedelta(days=1)

    # Write CSV file
    with open("speed.csv", "w") as file:
        file.write("date, provider, location, model, score\n")
        for row in rows:
            location = row[2] if row[2] else ""
            file.write(f"{row[0]}, {row[1]}, {location}, {row[3]}, {row[4]}\n")


def parse_model(model: str) -> tuple[str, str, str]:
    """
    Parse a model name into provider, location, and model strings
    """

    if "azure" in model:
        if "/" in model:
            [location, model] = model.split("/")
        else:
            [model, location, discard] = model.split(".")

        return (
            "azure",
            location.replace("fixie-", "").replace("azure", "").replace(".", ""),
            model.replace("fixie-", ""),
        )

    if "anthropic/" in model:
        return ("anthropic", None, model.split("/")[-1])
    elif model.startswith("claude-"):
        return ("anthropic", None, model)

    if "anyscale.com/" in model:
        return ("anyscale", None, model.split("/")[-1].lower())

    if "cloudflare/" in model or "cloudflare.com/" in model or "@cf/" in model:
        return ("cloudflare", None, model.split("/")[-1])

    if "cohere/" in model:
        return ("cohere", None, model.split("/")[-1])
    elif model.startswith("command-"):
        return ("cohere", None, model)

    if "databricks.com/" in model:
        return ("databricks", None, model.split("/")[-1])

    if "fireworks.ai/" in model:
        return ("fireworks", None, model.split("/")[-1])

    if "google/" in model:
        return ("google", None, model.split("/")[-1])

    if "groq.com/" in model:
        return ("groq", None, model.split("/")[-1])

    if model.startswith("Neets-"):
        return ("neets", None, model.lower())

    if "octo.ai/" in model or "octoai.run/" in model:
        return ("octoai", None, model.split("/")[-1])

    if "openai/" in model:
        return ("openai", None, model.split("/")[-1])
    elif model.startswith("gpt-"):
        return ("openai", None, model)

    if "ovh.net/" in model:
        return ("ovh", None, model.split("/")[-1])

    if "perplexity.ai/" in model:
        return ("perplexity", None, model.split("/")[-1])

    if (
        "together.ai/" in model
        or "together.xyz" in model
        or "togethercomputer/" in model
    ):
        return ("together", None, model.split("/")[-1])

    return ("?", None, model)


if __name__ == "__main__":
    download_all()
    score_all()
