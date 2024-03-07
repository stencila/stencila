---
# Test using `edit-code-chunk` to translate a document.
---
```python exec
# Currently, we use this to boot up the python kernel.
# But we should not need to do this in the future.
import sys
sys.version
```

// @edit-code-chunk translate this to python.
:::

```r exec
data <- read.table(
    'data.csv',
    header = TRUE,
    sep = ','
)
```
:::

// @edit-code-chunk translate this to python. Make it pretty though.
:::

```r exec
ggplot(data, aes(x=age, y=height)) + geom_point()
```

:::


// @edit-code-chunk turn this into R code.
:::
```python exec
import numpy as np
vals = np.linspace(-10, 10, 100)
lowest = np.min(np.abs(vals))
```
:::
