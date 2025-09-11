---
structuring:
- figures-with-captions
- tables-with-captions
- paragraphs-to-headings
---

This tests that bold figure/table captions are not incorrectly converted to headings. This is important to prevent caption paragraphs like "**Figure 1.** Caption text" from being misidentified as headings when ParagraphsToHeadings is also active.

**Figure 1.** This bold paragraph should be treated as a figure caption, not converted to a heading.

![Example image](image.png)

**Table 1.** This bold paragraph should be treated as a table caption, not converted to a heading.

| Data | Value |
|------|--------|
| A    | 1      |
| B    | 2      |

**Methods**

This bold paragraph should be converted to a heading because it's not a caption pattern and "Methods" is a valid section heading.

The caption detection should take precedence over heading conversion for properly formatted figure and table captions.