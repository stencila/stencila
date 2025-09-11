---
structuring:
  - heading1-to-title
  - headings-to-title
---

# The title

The above level 1 heading is the first content block, so Heading1ToTitle should extract it as title first.

## Overview

This level 2 heading should remain because Heading1ToTitle already extracted the title.

# Methodology

This second level 1 heading should remain because Heading1ToTitle only processes the very first block, and HeadingsToTitle should not extract it since a title was already set.
