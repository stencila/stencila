import * as schema from '@stencila/schema'
import { useContext } from 'solid-js'
import { Dynamic } from 'solid-js/web'
import { DocumentContext } from '../Document'

export function ImageObject(props: { node: schema.ImageObject }) {
  /**
   * Determine which component to use based on the content of the image
   */
  const component = () => {
    if (extractPlotly(props.node) !== undefined) {
      return ImageObjectPlotly
    }
    return ImageObjectStatic
  }
  return <Dynamic component={component()} node={props.node} />
}

export function ImageObjectStatic(props: { node: schema.ImageObject }) {
  const documentContext = useContext(DocumentContext)

  /**
   * Generate the `src` attribute of an `<img>` element from an
   * `ImageObject`'s `contentUrl` property (which may be relative to
   * the document it is in).
   */
  const src = () => {
    const contentUrl = props.node.contentUrl
    if (/^[a-z]{2,5}:\/\//.test(contentUrl)) return contentUrl

    let documentUrl = documentContext?.url ?? ''
    let parentUrl = documentUrl.slice(0, documentUrl.lastIndexOf('/'))
    return parentUrl + '/' + contentUrl
  }

  return <img itemtype="http://schema.org/ImageObject" src={src()}></img>
}

const plotlyMediaTypes = ['application/vnd.plotly.v1+json']

const extractPlotly = (
  image: schema.ImageObject
): [string, unknown] | undefined => {
  for (const node of image.content ?? []) {
    if (typeof node === 'object' && node !== null && 'mediaType' in node) {
      if (plotlyMediaTypes.includes(node.mediaType) && 'data' in node) {
        return [node.mediaType, node.data]
      }
    }
  }
}

export function ImageObjectPlotly(props: { node: schema.ImageObject }) {
  const elem = () => {
    const [mediaType, data] = extractPlotly(props.node) as [string, unknown]
    return (
      <stencila-image-plotly>
        <picture>
          <script type={mediaType}>{JSON.stringify(data)}</script>
        </picture>
      </stencila-image-plotly>
    )
  }
  return elem()
}
