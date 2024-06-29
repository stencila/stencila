import Levenshtein

STENCILA_MODELS = """
google/gemini-1.0-pro
google/gemini-1.0-pro-001
google/gemini-1.0-pro-latest
google/gemini-1.0-pro-vision-latest
google/gemini-1.5-flash
google/gemini-1.5-flash-001
google/gemini-1.5-pro
google/gemini-1.5-pro-001
google/gemini-pro
google/gemini-pro-vision
google/text-embedding-004
mistral/codestral-2405
mistral/codestral-latest
mistral/mistral-large-2402
mistral/mistral-large-latest
mistral/mistral-medium
mistral/mistral-medium-2312
mistral/mistral-medium-latest
mistral/mistral-small
mistral/mistral-small-2312
mistral/mistral-small-2402
mistral/mistral-small-latest
mistral/mistral-tiny
mistral/mistral-tiny-2312
mistral/open-mistral-7b
mistral/open-mixtral-8x22b
mistral/open-mixtral-8x22b-2404
mistral/open-mixtral-8x7b
openai/gpt-3.5-turbo
openai/gpt-3.5-turbo-0125
openai/gpt-3.5-turbo-0301
openai/gpt-3.5-turbo-0613
openai/gpt-3.5-turbo-1106
openai/gpt-3.5-turbo-16k
openai/gpt-3.5-turbo-16k-0613
openai/gpt-3.5-turbo-instruct
openai/gpt-3.5-turbo-instruct-0914
openai/gpt-4
openai/gpt-4-0125-preview
openai/gpt-4-0613
openai/gpt-4-1106-preview
openai/gpt-4-1106-vision-preview
openai/gpt-4-turbo
openai/gpt-4-turbo-2024-04-09
openai/gpt-4-turbo-preview
openai/gpt-4-vision-preview
openai/gpt-4o
openai/gpt-4o-2024-05-13
""".strip().split("\n")

SIMPLIFIED_MODELS = [model.split("/")[1] for model in STENCILA_MODELS]


def normalize_numbers(numbers: list[float | int | str]):
    floats = [float(num) for num in numbers]
    min_num = min(floats)
    max_num = max(floats)
    return [(float(num) - min_num) / (max_num - min_num) for num in floats]


def find_stencila_model(model_str: str, cutoff: float = 0.0) -> str | None:
    """Match to a stencila model."""
    # TODO: Not great, (we could get repeats). This will do for now
    distances = [Levenshtein.distance(model_str, model) for model in SIMPLIFIED_MODELS]
    min_distance = min(distances)
    if min_distance > cutoff:
        return None

    matches = [
        STENCILA_MODELS[i]
        for i, distance in enumerate(distances)
        if distance == min_distance
    ]
    return matches[0]
