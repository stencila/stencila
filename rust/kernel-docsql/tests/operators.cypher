test.cells(position = 1)
---
MATCH (cell:TableCell)
WHERE cell.position = 1
RETURN cell
LIMIT 10


test.cells(.position == 2)
---
MATCH (cell:TableCell)
WHERE cell.position = 2
RETURN cell
LIMIT 10


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
