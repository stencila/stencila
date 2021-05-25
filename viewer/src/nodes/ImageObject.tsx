import * as schema from '@stencila/schema'
import { useContext } from 'solid-js'
import { DocumentContext } from '../Document'

export function ImageObject(props: { node: schema.ImageObject }) {
  const documentContext = useContext(DocumentContext)

  const src = () => {
    const contentUrl = props.node.contentUrl
    if (/^[a-z]{2,5}:\/\//.test(contentUrl)) return contentUrl

    let documentUrl = documentContext?.url ?? ''
    let parentUrl = documentUrl.slice(0, documentUrl.lastIndexOf('/'))
    return parentUrl + '/' + contentUrl
  }

  return <img itemtype="http://schema.org/ImageObject" src={src()}></img>
}
