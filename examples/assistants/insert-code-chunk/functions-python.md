```python exec
def womble_size(age: int, diet: str) -> int:
    """Womble size depends on age and diet"""
    diet_multiplier = {
      'cheese': 1.1,
      'carrots': 0.5,
      'cheezels': 1.9,
    }.get(diet, 1.0)
    return age * 1.5 + multiplier * 3.0
```

// @insert-code-chunk get the size of a 5 year-old carrot eating womble.
