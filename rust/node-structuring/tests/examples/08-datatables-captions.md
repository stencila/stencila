---
structuring:
- tables-to-datatables
- tables-with-captions
---

This tests that simple tables with captions are properly converted to datatables while preserving caption association. This interaction is critical for structured data with descriptions - the caption must be correctly associated with the datatable, and only uniform simple tables should be converted.

Table 1. Performance metrics showing processing times and accuracy rates.

| Method | Time (ms) | Accuracy |
|--------|-----------|----------|
| A      | 120       | 95.2     |
| B      | 85        | 97.1     |
| C      | 203       | 89.7     |

The simple table above should be converted to a datatable because it has consistent rows, simple text-only cells, and no spans. The caption should be preserved and associated with the resulting datatable.
