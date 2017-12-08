title: Issues
author:
  - Oliver Buchtala
  - Nokome Bentley
type: Feature
status: Draft
---

# Introduction

> TODO

# Implementation

> TODO: refine

A simple formatter is defined like this:

```
cells: "C1:B10",
language: 'r',
apply: 'value < 0',
styles: {
  "background-color": "red"
}
```


```
cells: "C1:B10",
language: 'r',
apply: 'true',
styles: {
  "font-weight": "max(value * 100, 500)",
  "background-color": "hsv(sin(value),0.8,0.8)"
}
```

> we should us a consistent name for 'apply' internally, for the sake of consistency
