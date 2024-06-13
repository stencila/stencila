
def normalize_numbers(numbers: list[float | int | str]):
    floats = [float(num) for num in numbers]
    min_num = min(floats)
    max_num = max(floats)
    return [(float(num) - min_num) / (max_num - min_num) for num in floats]

