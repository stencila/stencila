---
# Test using `edit-code-chunk` to translate code.
---

// @edit-code-chunk translate this to R.
:::
```python exec
import pandas as pd

data = pd.read_csv('data.csv')
```
:::

// @edit-code-chunk turn this into R code.
:::
```python exec
import matplotlib.pyplot as plt
plt.figure(figsize=(10, 6))  # Set the figure size to be more 'pretty'
plt.scatter(data['age'], data['height'], edgecolor='k')
plt.title('Age vs Height')
plt.xlabel('Age')
plt.ylabel('Height')
plt.grid(True)
plt.show()
```
:::
