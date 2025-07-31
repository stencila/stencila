test.paragraphs().order_by('text', 'desc')
---
MATCH (paragraph:Paragraph)
RETURN paragraph
ORDER BY paragraph.text desc
LIMIT 10


test.figures().order_by('label')
---
MATCH (figure:Figure)
RETURN figure
ORDER BY figure.label
LIMIT 10
