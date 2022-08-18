"""
Union type for valid block content.
"""
BlockContent = Union["Call", "Claim", "CodeBlock", "CodeChunk", "Collection", "Figure", "Heading", "Include", "List", "MathBlock", "Paragraph", "QuoteBlock", "Table", "ThematicBreak"]
