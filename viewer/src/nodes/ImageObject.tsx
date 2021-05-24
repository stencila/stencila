import * as schema from '@stencila/schema'

export function ImageObject(props: { node: schema.ImageObject }) {
  return (
    <img
      itemtype="https://schema.org/ImageObject"
      src={props.node.contentUrl}
    ></img>
  )
}
