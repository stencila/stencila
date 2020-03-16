"""
Union type for valid block content.
"""
BlockContent = Union["CodeBlock", "CodeChunk", "Collection", "Figure", "Heading", "List", "ListItem", "MathBlock", "Paragraph", "QuoteBlock", "Table", "ThematicBreak"]
