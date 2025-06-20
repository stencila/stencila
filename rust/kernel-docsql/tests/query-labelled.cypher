table(1)
---
MATCH (`table`:`Table`)
WHERE `table`.label = '1'
RETURN `table`
LIMIT 1


figure(1)
---
MATCH (figure:Figure)
WHERE figure.label = '1'
RETURN figure
LIMIT 1


equation(1)
---
MATCH (equation:Equation)
WHERE equation.label = '1'
RETURN equation
LIMIT 1


figure(above)
---
MATCH (figure:Figure)
WHERE figure.position<$currentPosition
RETURN figure
ORDER BY figure.position DESC
LIMIT 1


tables()
---
MATCH (`table`:`Table`)
RETURN `table`
LIMIT 10


figures(above).limit(2)
---
MATCH (figure:Figure)
WHERE figure.position<$currentPosition
RETURN figure
ORDER BY figure.position DESC
LIMIT 2


equations()
---
MATCH (equation:Equation)
RETURN equation
LIMIT 10






