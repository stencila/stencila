"""
Union type for valid block content.
"""
BlockContent = Union["CodeBlock", "CodeChunk", "Heading", "List", "ListItem", "Paragraph", "QuoteBlock", "Table", "ThematicBreak"]
