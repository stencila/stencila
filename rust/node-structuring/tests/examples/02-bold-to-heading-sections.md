---
structuring:
- paragraphs-to-headings
- headings-to-sections
---

This tests that bold paragraphs like **Methods** are first converted to headings with appropriate level (level 1 for primary sections), then wrapped in section elements. This interaction is important because the heading level determination affects section hierarchy and primary sections must be recognized correctly.

**Introduction**

The bold paragraph above should be converted to a level 1 heading because "Introduction" is a primary section type, then wrapped in a section element.

**Methods**

This bold paragraph should also become a level 1 heading since "Methods" is a primary section type, then be wrapped in a section.

**Data Analysis**

This bold paragraph should become a heading (level 2 since it's not a primary section type) and then be wrapped appropriately in the section hierarchy.