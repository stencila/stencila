---
structuring:
- headings-to-paragraphs
- paragraphs-to-headings
---

This tests bidirectional conversion where long headings become paragraphs while bold paragraphs become headings. This interaction must not create conflicts or infinite loops, and the operations must apply their criteria correctly to avoid interfering with each other.

# This is a very long heading that exceeds 80 characters and should be converted to a paragraph because it exhibits paragraph-like traits rather than proper heading characteristics.

# This heading ends with a period and should also be converted to a paragraph.

**Methods**

The bold paragraph above should be converted to a heading because it's short, has no punctuation, and represents a valid section name.

**Results and Analysis**

This bold paragraph should also become a heading since it meets the criteria for heading conversion.

This demonstrates that the operations can work together: inappropriate headings become paragraphs while appropriate bold text becomes headings, without creating conflicts between the two transformations.