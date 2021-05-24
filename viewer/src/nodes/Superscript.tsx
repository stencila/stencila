import * as schema from '@stencila/schema'
import { InlineContentArray } from './InlineContent'

export function Superscript(props: { node: schema.Superscript }) {
  return (
    <sup itemtype="https://schema.stenci.la/Superscript">
      <InlineContentArray nodes={props.node.content}></InlineContentArray>
    </sup>
  )
}
