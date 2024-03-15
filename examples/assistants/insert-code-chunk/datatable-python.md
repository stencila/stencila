---
# Test `insert-code-chunk` makes use of data tables loaded into the kernel.
---

```python exec
import pandas as pd

data = pd.read_csv('data.csv')
```

// @insert-code-chunk plot data
