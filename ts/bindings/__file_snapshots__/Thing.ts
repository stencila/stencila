/**
 * The most generic type of item.
 */
export interface Thing extends Entity {
  type: 'Thing' | 'Article' | 'AudioObject' | 'Brand' | 'Collection' | 'ContactPoint' | 'CreativeWork' | 'Datatable' | 'DatatableColumn' | 'Figure' | 'ImageObject' | 'MediaObject' | 'Organization' | 'Periodical' | 'Person' | 'Product' | 'PublicationIssue' | 'PublicationVolume' | 'SoftwareApplication' | 'SoftwareEnvironment' | 'SoftwareSession' | 'SoftwareSourceCode' | 'Table' | 'VideoObject' | 'VolumeMount'
  alternateNames?: Array<string>
  description?: string | Array<Node>
  name?: string
  url?: string
}

/**
 * Create a `Thing` node
 * @param options Optional properties
 * @returns {Thing}
 */
export const thing = (
  options: OptionalProps<Thing> = {}
): Thing => ({
  ...(compact(options)),
  type: 'Thing'
})

