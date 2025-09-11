---
structuring:
- remove-empty-headings
- remove-empty-lists
---

This tests that all types of empty elements are removed in the correct order. This interaction is important to ensure cleanup operations work together without interfering and that the document is properly cleaned of all empty content while preserving valid elements.

# Valid Heading

# 

The empty heading above should be removed.

This paragraph should remain.

##

Another empty heading that should be removed.

-

The empty list above should be removed.

Final paragraph that should remain to show that valid content is preserved while all empty elements (headings, paragraphs, list items, and whitespace-only text) are properly removed.