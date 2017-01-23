# A simple test document

Limited to pretty much what can be represented with basic Markdown. It borrows text and examples from http://daringfireball.net/projects/markdown/syntax.text.

## Block elements

### Paragraphs

Like this one.

And this one, which is longer. Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

### Headers

Atx-style headers are used as the default. They use 1-6 hash characters at the start of the line, corresponding to header levels 1-6

# H1

## H2

### H3

#### H4

##### H5

###### H6

### Blockquotes

Blockquotes use a `>` before every line (not the "lazy" single `>`):

> This is a blockquote with two paragraphs. Lorem ipsum dolor sit amet,
> consectetuer adipiscing elit. Aliquam hendrerit mi posuere lectus.
> Vestibulum enim wisi, viverra nec, fringilla in, laoreet vitae, risus.
> 
> Donec sit amet nisl. Aliquam semper ipsum sit amet velit. Suspendisse
> id sem consectetuer libero luctus adipiscing.

Blockquotes can contain other Markdown elements, including headers, lists, and code blocks:

> ## This is a header.
> 
> 1.   This is the first list item.
> 2.   This is the second list item.
> 
> Here's some example code:
> 
>     return shell_exec("echo $input | $markdown_script");

### Lists

Unordered lists use hyphens:

- Red
- Green
- Blue

Ordered lists use numbers followed by periods:

1. One
2. Two
3. Three

List items may consist of multiple paragraphs. Each subsequent
paragraph in a list item is indented by 4 spaces:

1.  This is a list item with two paragraphs. Lorem ipsum dolor
    sit amet, consectetuer adipiscing elit. Aliquam hendrerit
    mi posuere lectus.

    Vestibulum enim wisi, viverra nec, fringilla in, laoreet
    vitae, risus. Donec sit amet nisl. Aliquam semper ipsum
    sit amet velit.

2.  Suspendisse id sem consectetuer libero luctus adipiscing.

### Code blocks

Fenced code blocks are used:

```
Some code
```

The language can be specified:

```r
Some R code
```

### Horizontal rules

Three hyphens are used for a horizontal rule:

---

## Span elements

### Links

This is [an example link](http://example.com/) and [this one also has a title](http://example.com/ "Title").

Autolinks are supported e.g. http://example.com

### Emphasis

Emphasis (HTML5 em) uses single underscore: _emphasized_.

### Importance

Importance (HTML5 strong) uses single asterisks: *important*.

### Code

Inline code uses `backticks`.

### Images

Inline image syntax looks like this: ![Alt text](/path/to/img.jpg)

Optionally, you can include a title: ![Alt text](/path/to/img.jpg "Optional title")


