test
---
MATCH (node)
RETURN *
LIMIT 10


test.tables()
---
MATCH (`table`:`Table`)
RETURN `table`
LIMIT 10


test.cells().skip(3).limit(2)
---
MATCH (cell:TableCell)
RETURN cell
SKIP 3
LIMIT 2
