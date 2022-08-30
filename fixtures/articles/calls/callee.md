&[n]{num min=1 def=100}

&[color]{enum vals=["red","blue","green"]}

```r exec
hist(rnorm(n), col=color, breaks=50, main="")
```

Figure 1. A histogram of `unbox(n)`{r exec} random numbers drawn from the standard normal distribution.
