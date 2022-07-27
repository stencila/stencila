This example illustrates the use of several node types that are encoded as RPNGs in formats such as Microsoft Word and Google Docs: `Parameter`, `CodeChunk`, `CodeExpression`, `MathFragment`, and `MathBlock`.

The probability density function for the Normal distribution is,

$$
f(x) = \frac{1}{\sigma\sqrt{2\pi}}\exp\left( -\frac{1}{2}\left(\frac{x-\mu}{\sigma}\right)^{\!2}\right)
$$

Change the following parameters to generate random numbers drawn from the Normal distribution or other distributions.

The probability distribution: /dist/{type=enum default=Normal values=['Normal', 'Lognormal', 'Poisson']}

The sample size: /n/{type=number default=100 min=1 max=10000}.

The number of histogram breaks: /breaks/{type=number default=30 min=3 max=100}.

```r exec
rfunc <- switch(dist,
    Normal = rnorm,
    Lognormal = rlnorm,
    Poisson = function(n) rpois(n, 1)
)
rands <- rfunc(n)
hist(rands, main = paste(dist, "distribution"), breaks = breaks, col = 'blue')
```

The mean ($\mu$) of the sample is `unbox(mean(rands))`{r exec} and its standard deviation ($\sigma$) is `unbox(sd(rands))`{r exec}.
