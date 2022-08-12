An example mainly intended for testing that cancellation of both individual, and all, `CodeChunk` and `CodeExpression` nodes in a document works correctly across different language kernels.

# Python

A Python code chunk that sleeps for a while:

```py exec
import os,time
time.sleep(1)
py_pid = os.getpid()
```

A Python code chunk that is dependent on the first. It prints the PID of the main kernel process, which may differ to the current if being executed in a fork:

```py exec
[py_pid, os.getpid()]
```

A Python code expression that has the same code as the previous code chunk: `[py_pid, os.getpid()]`{py exec}.

# R

A R code chunk that sleeps for a while:

```r exec
Sys.sleep(1)
r_pid <- Sys.getpid()
```

A R code chunk that is dependent on the first:

```r exec
c(r_pid, Sys.getpid())
```

A R code expression that has the same code as the previous code chunk: `c(r_pid, Sys.getpid())`{r exec}.
