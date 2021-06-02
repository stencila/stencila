"""
Union type for valid block content.
"""
BlockContent = Union["Claim", "CodeBlock", "CodeChunk", "Collection", "Figure", "Heading", "List", "MathBlock", "Paragraph", "QuoteBlock", "Table", "ThematicBreak"]
