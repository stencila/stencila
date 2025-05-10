test.cells(.position < 3)
---
MATCH (cell:TableCell)
WHERE cell.position < 3
RETURN cell
LIMIT 10


test.cells(.text != 'a')
---
MATCH (cell:TableCell)
WHERE cell.text <> 'a'
RETURN cell
LIMIT 10


test.cells(.text =~ 'a')
---
MATCH (cell:TableCell)
WHERE regexp_matches(cell.text, 'a')
RETURN cell
LIMIT 10


test.cells(.text !~ 'a')
---
MATCH (cell:TableCell)
WHERE NOT regexp_matches(cell.text, 'a')
RETURN cell
LIMIT 10


test.cells(.text ^= 'a')
---
MATCH (cell:TableCell)
WHERE starts_with(cell.text, 'a')
RETURN cell
LIMIT 10


test.cells(.text $= 'a')
---
MATCH (cell:TableCell)
WHERE ends_with(cell.text, 'a')
RETURN cell
LIMIT 10


test.cells(.text in ['a', 'b'])
---
MATCH (cell:TableCell)
WHERE list_contains(["a", "b"], cell.text)
RETURN cell
LIMIT 10


test.cells(.text has 'a')
---
MATCH (cell:TableCell)
WHERE list_contains(cell.text, 'a')
RETURN cell
LIMIT 10


test.paragraphs(search = 'keyword')
---
CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword')
RETURN node
LIMIT 10


test.paragraphs(searchAll = 'keyword1 keyword2')
---
CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword1 keyword2', conjunctive := true)
RETURN node
LIMIT 10


test.paragraphs(.text ^= 'Word', search = 'keyword')
---
CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword')
WHERE starts_with(node.text, 'Word')
RETURN node
LIMIT 10
