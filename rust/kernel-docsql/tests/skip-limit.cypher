test.cells().skip(3).limit(2)
---
MATCH (cell:TableCell)
RETURN cell
SKIP 3
LIMIT 2
