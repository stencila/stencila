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
