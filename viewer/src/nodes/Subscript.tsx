import * as schema from '@stencila/schema'
import { InlineContentArray } from './InlineContent'

export function Subscript(props: { node: schema.Subscript }) {
  return (
    <sub itemtype="http://schema.stenci.la/Subscript">
      <InlineContentArray nodes = {props.node.content}></InlineContentArray>
    </sub>
  )
}
