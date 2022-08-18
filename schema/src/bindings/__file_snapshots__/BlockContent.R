#' Union type for valid block content.
#'
#' @return A `list` of class `Union` describing valid subtypes of this type
#' @export
BlockContent <- Union(Call, Claim, CodeBlock, CodeChunk, Collection, Figure, Heading, Include, List, MathBlock, Paragraph, QuoteBlock, Table, ThematicBreak)

