/**
 * The most generic type of item https://schema.org/Thing.
 */
export interface Thing  {
  type: 'Thing' | 'CreativeWork' | 'Article' | 'MediaObject' | 'AudioObject' | 'Brand' | 'Code' | 'CodeBlock' | 'SoftwareSourceCode' | 'CodeChunk' | 'CodeExpr' | 'Collection' | 'ContactPoint' | 'Datatable' | 'DatatableColumn' | 'DatatableColumnSchema' | 'Mark' | 'Delete' | 'Emphasis' | 'Environment' | 'Heading' | 'ImageObject' | 'Include' | 'Link' | 'List' | 'ListItem' | 'Mount' | 'Organization' | 'Paragraph' | 'Person' | 'Product' | 'Quote' | 'QuoteBlock' | 'ResourceParameters' | 'SoftwareApplication' | 'SoftwareSession' | 'Strong' | 'Subscript' | 'Superscript' | 'Table' | 'TableCell' | 'TableRow' | 'ThematicBreak' | 'VideoObject'
  alternateNames?: Array<string>
  description?: string
  id?: string
  meta?: {[key: string]: any}
  name?: string
  url?: string
}

/**
 * Create a `Thing` node
 * @param options Optional properties
 * @returns {Thing}
 */
export const thing = (
  options: {
    alternateNames?: Array<string>
    description?: string
    id?: string
    meta?: {[key: string]: any}
    name?: string
    url?: string
  } = {}
): Thing => ({
  ...options,
  type: 'Thing'
})

