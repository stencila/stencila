An example article mainly intended for testing that `Datatable` variables can be passed between different kernels.

# Setting

These code blocks set `Datatable` variables in each kernel:

```py exec
import time
time.sleep(9)
```

```r exec
r <- data.frame(a=1:10,b=2:11)
```

# Getting

These code blocks get the variables from the other kernels:

```py exec
r.head()
```

```r exec
head(py)
```
