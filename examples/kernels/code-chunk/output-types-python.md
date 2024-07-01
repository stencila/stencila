The `python` kernel supports outputting most primitive node types. The `print` function is patched so that each argument is a separate output:

```python exec
print(True, 1, 2.34, "string", [1, 2, 3], { "a": 1, "b": 2 })
```

There is also support for outputting of `numpy` arrays:

```python exec
import numpy as np

a1 = np.array([True, False], dtype=np.bool_)
a2 = np.array([-1, 0, 1], dtype=np.int_)
a3 = np.array([1, 2 , 3], dtype=np.uint)
a4 = np.array([1.23, 4.56], dtype=64)

print(a1, a2, a3, a4)
```

And `pandas` data frames:

```python exec
import pandas as pd

df = pd.DataFrame({
    'c1': [True, False],
    'c2': [1, 2],
    'c3': [1.23, 4.56],
    'c4': ['One', 'Two']
})

df
```

And `matplotlib` plots:

```python exec
import matplotlib.pyplot as plt

plt.plot([1, 2, 3, 4], [1, 2, 4, 3])
```
