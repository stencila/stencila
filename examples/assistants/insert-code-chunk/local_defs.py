import numpy as np
import pandas as pd
from dataclasses import dataclass

a_dataframe = pd.read_csv("data.csv")
an_array = np.array([1, 2, 3, 4, 5], dtype=float)
an_integer = 666  # the number of the beast
a_string = "hello, world"
a_boolean = True
a_none = None
a_list = [str(n) for n in range(5)] + [None] + ["blarg"]
a_dict = {"a": 1, "b": 2, "c": 3}


@dataclass
class Planet:
    name: str
    mass: int
    distance_from_sun: float


shape = ["cylinder", "cone", "sphere"]

p = Planet(name="Venus", mass=4.87e24, distance_from_sun=108e6)


def escape_velocity(p: Planet, rocket_shape: str) -> float:
    return 11.2 * np.sqrt(p.mass / rocket_shape)
