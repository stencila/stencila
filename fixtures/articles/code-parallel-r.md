This fixture is focussed on testing scheduling of parallel execution plans with R.
It is very similar to the sibling fixture, `code-parallel-python.md` but implemented in R, allowing us to check for consistency in behaviour of parallel execution between the two and help isolate any bugs.

Stage 1, a single setup chunk:

```r exec
show_time <- function(curr, prev_time, prev) {
    paste0(
        "Chunk ", curr, " succeeded at ", Sys.time(),
        ", ", Sys.time()-prev_time, "s after chunk ", prev
    )
}

Sys.sleep(1)
chunk1 <- Sys.time()
paste("Chunk 1 succeeded at", chunk1)
```

Stage 2, three code chunks that should start at the same time and execute in parallel but with different durations.

```r exec
Sys.sleep(1)
show_time(2, chunk1, 1)
```

```r exec
Sys.sleep(2)
show_time(3, chunk1, 1)
```

```r exec
Sys.sleep(3)
show_time(4, chunk1, 1)
```

Stage 3, should begin after chunks 2-4 have finished.

```r exec
Sys.sleep(1)
chunk5 <- Sys.time()
show_time(5, chunk1, 1)
```

Stage 4, three chunks that should run in parallel after chunk 5 and finish about 1 second after it.

```r exec
Sys.sleep(1)
show_time(6, chunk5, 5)
```

```r exec
Sys.sleep(1)
show_time(7, chunk5, 5)
```

```r exec
Sys.sleep(1)
show_time(8, chunk5, 5)
```
