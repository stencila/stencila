# Page Heading 1

First block of text under first heading

:::{prf:theorem}
### Maxwell's equations

In partial differential equation form and SI units, Maxwell's microscopic equations can be written as

```{math}
:label: maxwell
\begin{align}\nabla \cdot \mathbf{E} \,\,\, &= \frac{\rho}{\varepsilon_0}\\\nabla \cdot \mathbf{B} \,\,\, &= 0\\\nabla \times \mathbf{E} &= -\frac{\partial \mathbf{B}}{\partial t}\\\nabla \times \mathbf{B} &= \mu_0 \left(\mathbf{J} + \varepsilon_0  \frac{\partial \mathbf{E}}{\partial t} \right)\end{align}
```
:::

```{code-cell} python
hello = "hello"
there = "there"
phrase = f"{hello}, {there}!"
print(phrase)

```

## An Embedded Include block

```{embed} #maxwell
```

## A Quote Block

:::{blockquote}
We know what we are, but know not what we may be.

-- Hamlet act 4, Scene 5
:::

## Section Block

+++ { "part": "summary" }
MyST makes Markdown more _extensible_ & **powerful** to support an
ecosystem of tools for computational narratives, technical documentation,
and open scientific communication. You can **edit this demo** including the [frontmatter](https://mystmd.org/guide/frontmatter) to change the title!!

:::{important} Our Values
We believe in a community-driven approach of open-source tools that are
composable and extensible. You can find out how to be involved in developing MyST Markdown by getting involved in the [ExecutableBooks](https://executablebooks.org/) project.

:::

+++

