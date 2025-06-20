variable('a')
---
MATCH (variable:Variable)
WHERE variable.name = 'a'
RETURN variable
LIMIT 1


variables()
---
MATCH (variable:Variable)
RETURN variable
LIMIT 10


variables(nodeType = 'Integer')
---
MATCH (variable:Variable)
WHERE variable.nodeType = 'Integer'
RETURN variable
LIMIT 10


variables(above, limit = 2)
---
MATCH (variable:Variable)
WHERE variable.position<$currentPosition
RETURN variable
ORDER BY variable.position DESC
LIMIT 2


variables(below).limit(2)
---
MATCH (variable:Variable)
WHERE variable.position>$currentPosition
RETURN variable
ORDER BY variable.position
LIMIT 2
