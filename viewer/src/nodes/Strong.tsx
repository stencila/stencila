import * as schema from '@stencila/schema'
import { InlineContentArray } from './InlineContent'

export function Strong(props: { node: schema.Strong }) {
  return (
    <strong itemtype="https://schema.stenci.la/Strong">
      <InlineContentArray nodes = {props.node.content}></InlineContentArray>
    </strong>
  )
}
