---
title: An article with executable nodes within list items
description: |
  This article illustrates the placement of executable node types such as `Parameter` and `CodeExpression` within list items. It is mainly used for testing that these more deeply nodes are actually executed.
---

- Some parameters:

  - /a/{type=num default=1}
  - /b/{type=num default=2}

- Some code expressions:
  - `a + b`{calc exec}
  - And a nested ordered list
    1. `a * b`{calc exec}
    2. `a / b`{calc exec}
