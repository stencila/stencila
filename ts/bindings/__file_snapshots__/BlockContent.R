#' Union type for valid block content.
#'
#' @return A `list` of class `Union` describing valid subtypes of this type
#' @export
BlockContent <- Union(Claim, CodeBlock, CodeChunk, Collection, Figure, Heading, List, MathBlock, Paragraph, QuoteBlock, Table, ThematicBreak)

