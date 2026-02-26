# Stencila walkthroughs

Walkthrough are used inside of the Stencila VCode Extension as interactive ways for
Stencila users to learn about and trial functionality of Stencila.

They are written in Stencila Markdown and are meant to be opened up progressively in sections with each section separated by `...`. These `...`s become "Continue ->" buttons when the user views them as walkthroughs and can move through them step-by-step.

Using section breaks, you can write user onboarding documents that are able to be
revealed and "walked" through.

## Conventions

### Syntax

When writing a walkthrough, you'll write in Stencila Markdown, but Stencila Users may be interested in walkthroughs in other flavors of Markdown such as MyST or Quarto. This means you should write fence blocks in Stencila Markdown style, and these will be converted to MyST and Quarto Markdown using the Makefile in this directory.

Example:

```markdown
::: create @figure-code a python plot with 1000 random numbers :::
```

Would be automatically converted to MyST Syntaxx of when the `Makefile` is run against it:

```markdown
:::{create} a python plot with 1000 random numbers
:prompt: @figure-code
:::
```

The automatic conversion will only work for native blocks. It will not work inside of code blocks. So please refrain from writing walkthroughs that place examples or syntax inside of code blocks. A preffered pattern is to show the user an example block and ask them to replicate / transfer their understanding to writing their own.

### Learning Objectives

We try to make these walkthrougha as accessible as possible, so if you're planning to write a walkthrough, plese think about what learning objectives you have for the reader, and how you'll lay them out over the course of the walkthrough. Then at athe end, how you will recap those objectvices so the reader can close-out and better understand. Using a chart of Bloom's Taxonomy words [such as this one](https://www.flickr.com/photos/vandycft/29428436431) can help you write appropriate learning objectives and recaps for the types of skill's you're trying to teach in a walkthrough.

## Building Walkthroughs

Walkthroughs should be written in `.smd` and added to the top of the `Makefile`
in this directory. Not all `.smd`s are built by the `Makefile` as they can't all
be converted to other forms of Markdown. You should add the new `.smd` file without its
extension to the `SOURCES` variable in the `Makefile` then run `make` in this directory.
