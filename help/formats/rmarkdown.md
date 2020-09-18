---
id: 4458566
title: R Markdown
description: Authoring executable documents with R Markdown
published: true
---

R Markdown is a popular format for reproducible research. This article provides a guide to how to write, and collaborate on, a reproducible research article using R Markdown and Stencila. As well as describing the basics of embedding R code in your article, it covers how to include meta-data (such as author names and affiliations), citations and references. Finally, we discuss how you can preview your article and convert it to other formats using Stencila.

# Introduction

[R Markdown](https://rmarkdown.rstudio.com/) is a popular format for writing reproducible documents. [Markdown](https://daringfireball.net/projects/markdown/syntax) is a simple text format, which, in the words of it's creator, is "intended to be as easy-to-read and easy-to-write as is feasible". R Markdown extends Markdown with some custom syntax for embedding executable R code.

There are many resources available for how to use R Markdown for reproducible research, including:

- Chris Hartgerink (2017) _Composing reproducible manuscripts using R Markdown_ https://elifesciences.org/labs/cad57bcf/composing-reproducible-manuscripts-using-r-markdown

- Daniel Nüst, Vicky Steeves, Rémi Rampin, Markus Konkol, Edzer Pebesma (2018)_Writing reprocible geoscience papers using R Markdown, Docker, and GitLab_ https://vickysteeves.gitlab.io/repro-papers/index.html

- Paul Bauer (2018) _Writing a Reproducible Paper in R Markdown_ https://papers.ssrn.com/sol3/papers.cfm?abstract_id=3175518

# Code chunks

## The basics

At the heart of R Markdown are "code chunks". Code chunks can produce outputs, or they can be used to execute arbitrary R code. For example, here we assign a variable called `randoms` which will be an array of 1000 random numbers sampled from a normal distribution:

````md
```{r}
randoms <- rnorm(1000)
```
````

When you assign a variable in a code chunk you can reuse it in another chunk, later in the document. For example, we can plot the frequency distribution of the random numbers we assigned to the `randoms` variable earlier:

````md
```{r}
hist(randoms, breaks=30, col="grey", main="")
```
````

## Adding figure and table captions

R Markdown allows you to specify "options" for code chunks. One such option is `fig.cap` which allows you to specify a figure caption:

````md
```{r fig.cap="My figure"}
plot(randoms)
```
````

That is OK for short captions, but when you have longer captions you will probably want to use Bookdown style text references e.g.

````md
```{r fig.cap="(ref:figure3)"}
plot(randoms)
```

(ref:figure3) This is a slightly longer caption for my figure including some **strong** emphasis.
````

Bookdown style text references are good for longer, single paragraph captions. However, for more structured captions having a title and paragraph as is often found in academic journals we suggest that you use a code chunk block extension. e.g.

````md
chunk: Figure 3
:::

### The title of my plot

A paragraph for my figure including some **strong** emphasis.

```{r}
plot(randoms)
```

:::
{#fig3}
````

# Inline code chunks

In R Markdown you can also inline "inline" code chunks within paragraphs and other text content. For example, here is the mean of the `randoms` variable that we assigned earlier: `r mean(randoms)`. In Stencila, we call these `CodeExpression`s, because they are intended to display a calculated value, and shouldn't be used for statements, such as assigning a variable.
