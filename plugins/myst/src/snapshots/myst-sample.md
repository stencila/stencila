```yaml
title: Working with MyST Markdown
subtitle: A live demo
authors:
  - name: Rowan Cockett
    orcid: 0000-0002-7859-8394
    affiliations:
      - Executable Books
license: CC-BY-4.0
```

MyST makes Markdown more _extensible_ & **powerful** to support an
ecosystem of tools for computational narratives, technical documentation,
and open scientific communication. You can **edit this demo** including the [frontmatter](https://mystmd.org/guide/frontmatter) to change the title!!

:::{important} Our Values
We believe in a community-driven approach of open-source tools that are
composable and extensible. You can find out how to be involved in developing MyST Markdown by getting involved in the [ExecutableBooks](https://executablebooks.org/) project.

:::

MyST allows you to create figures with rich cross-references, scientific citations, and export to many commonly used document formats, including ([websites like this one](https://mystmd.org/guide/quickstart-myst-websites), [PDFs & {math}`\LaTeX`](https://mystmd.org/guide/creating-pdf-documents), [Microsoft Word](https://mystmd.org/guide/creating-word-documents), and [JATS XML](https://mystmd.org/guide/creating-jats-xml)).

For example, we have included a figure below ([](#my-fig)), [](#example-table) as well as [](#maxwell), a cross-reference to Maxwell’s equations.
You can click on these and see the preview of the reference immediately.

## Including Figures and Images

:::{figure}
:name: my-fig
![](https://source.unsplash.com/random/400x200?beach,ocean)

Relaxing at the beach 🏝 🌊 😎

:::

## Including Math and Equations

```{math}
:label: maxwell
\begin{aligned}
\nabla \times \vec{e}+\frac{\partial \vec{b}}{\partial t}&=0 \\
\nabla \times \vec{h}-\vec{j}&=\vec{s}\_{e}
\end{aligned}
```
## Including Tables

:::{table}
:name: example-table
This is a nice table!

| Training  | Validation |
|--- |--- |
| 0  | 5 |
| 13720  | 2744 |

:::

## Callouts

:::{note} Note
:class: dropdown
This is initially hidden, and can be clicked to be opened when you are viewing the content.

:::

