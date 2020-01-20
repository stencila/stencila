"""
Union type for valid block content.
"""
BlockContent = Union["CodeBlock", "CodeChunk", "Heading", "List", "ListItem", "MathBlock", "Paragraph", "QuoteBlock", "Table", "ThematicBreak"]
