"""
Module for downloading and processing model cost metrics collated by LiteLLM
at https://github.com/BerriAI/litellm/blob/main/model_prices_and_context_window.json
"""

import json
from os import path, makedirs

import requests

# Local directory with downloaded files
downloads = path.join(path.dirname(path.abspath(__file__)), "downloads", "litellm")
makedirs(downloads, exist_ok=True)

file_name = "model_prices_and_context_window.json"
file_path = path.join(downloads, file_name)


def download():
    """
    Download current version of data file
    """

    url = "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json"
    response = requests.get(url, stream=True)
    response.raise_for_status()

    print(f"Downloading {file_name}")
    with open(file_path, "wb") as file:
        for chunk in response.iter_content(chunk_size=8192):
            if chunk:
                file.write(chunk)


def score():
    """
    Extract input and output costs and derive a score for each model
    """

    with open(file_path, "r") as file:
        data = json.load(file)

    # Extract values and calculate cost
    rows = []
    min = 1e6
    max = 0
    for model, details in data.items():
        values = extract(model, details)
        if values:
            rows.append(values)
            cost = values[4]
            if cost < min:
                min = cost
            if cost > max:
                max = cost

    # Normalize scores by min and max where 1=cheapest and 0=most expensive
    range = max - min
    for index, row in enumerate(rows):
        rows[index][4] = round((max - rows[index][4]) / range, 6)

    # Write CSV file
    with open("cost.csv", "w") as file:
        file.write("provider, model, input, output, score\n")
        for row in rows:
            [provider, model, input, output, score] = row
            file.write(f"{provider}, {model}, {input}, {output}, {score}\n")


def extract(model: str, details: dict) -> tuple[str, str, float, float, float]:
    """
    Extract values (and filter out unwanted models)
    """
    mode = details.get("mode")
    if mode not in ["chat"]:
        return None

    provider = details.get("litellm_provider")
    input = details.get("input_cost_per_token")
    output = details.get("output_cost_per_token")

    if (
        provider is None
        or input is None
        or output is None
        or (input == 0 and output == 0)
    ):
        return None

    model = model.split("/")[-1].lower()

    if provider == "openai" and model.startswith("ft:"):
        return None

    if provider == "cohere_chat":
        provider = "cohere"
    elif provider.startswith("vertex_ai"):
        provider = "vertex"

    return [
        provider,
        model,
        round(input * 1e6, 4),
        round(output * 1e6),
        round((input + output) * 1e6),
    ]


if __name__ == "__main__":
    # download()
    score()
