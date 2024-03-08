import numpy as np
import pandas as pd

a_dataframe = pd.read_csv("data.csv")
an_array = np.array([1, 2, 3, 4, 5], dtype=float)
an_integer = 666  # the number of the beast
a_string = "hello, world"
a_boolean = True
a_none = None
a_list = [str(n) for n in range(5)] + [None] + ["blarg"]
a_dict = {"a": 1, "b": 2, "c": 3}


def a_function(x: int, name: str) -> str:
    """A function that takes an integer and a string and returns a string."""
    return f"Hello, {name}! {x} is a number."
