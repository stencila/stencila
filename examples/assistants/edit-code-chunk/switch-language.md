---
# Test using `edit-code-chunk` to translate a document.
---

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
