test.paragraphs(@above)
---
MATCH (paragraph:Paragraph)
WHERE paragraph.position<$currentPosition
RETURN paragraph
ORDER BY paragraph.position DESC
LIMIT 10


test.figures(@below)
---
MATCH (figure:Figure)
WHERE figure.position>$currentPosition
RETURN figure
ORDER BY figure.position
LIMIT 10
