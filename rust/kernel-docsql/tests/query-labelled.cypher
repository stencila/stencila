table(1)
---
MATCH (`table`:`Table`:CodeChunk)
WHERE `table`.label = '1'
  AND (starts_with(table.nodeId, 'tab') OR table.labelType = 'TableLabel')
RETURN `table`
LIMIT 1


figure(1)
---
MATCH (figure:Figure:CodeChunk)
WHERE figure.label = '1'
  AND (starts_with(figure.nodeId, 'fig') OR figure.labelType = 'FigureLabel')
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
MATCH (figure:Figure:CodeChunk)
WHERE figure.position<$currentPosition
  AND (starts_with(figure.nodeId, 'fig') OR figure.labelType = 'FigureLabel')
RETURN figure
ORDER BY figure.position DESC
LIMIT 1


tables()
---
MATCH (`table`:`Table`:CodeChunk)
WHERE (starts_with(table.nodeId, 'tab') OR table.labelType = 'TableLabel')
RETURN `table`
LIMIT 10


figures(above).limit(2)
---
MATCH (figure:Figure:CodeChunk)
WHERE figure.position<$currentPosition
  AND (starts_with(figure.nodeId, 'fig') OR figure.labelType = 'FigureLabel')
RETURN figure
ORDER BY figure.position DESC
LIMIT 2


equations()
---
MATCH (equation:Equation)
RETURN equation
LIMIT 10
