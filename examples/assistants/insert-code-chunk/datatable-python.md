---
# Test `insert-code-chunk` makes use of data tables loaded into the kernel.
---

In this first code chunk, we read a CSV file using Python,

```python exec
import pandas as pd

data = pd.read_csv('data.csv')
```

In the background a "hint" to the structure of the data is made available to assistants within system prompts. 

::: do @insert-code-chunk plot data
