/**
 * The most generic type of item.
 */
export interface Thing extends Entity {
  type: 'Thing' | 'Article' | 'AudioObject' | 'Brand' | 'Code' | 'CodeBlock' | 'CodeChunk' | 'CodeExpr' | 'Collection' | 'ContactPoint' | 'CreativeWork' | 'Datatable' | 'DatatableColumn' | 'Environment' | 'ImageObject' | 'MediaObject' | 'Mount' | 'Organization' | 'Periodical' | 'Person' | 'Product' | 'PublicationIssue' | 'PublicationVolume' | 'ResourceParameters' | 'SoftwareApplication' | 'SoftwareSession' | 'SoftwareSourceCode' | 'Table' | 'VideoObject'
  alternateNames?: Array<string>
  description?: string
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

