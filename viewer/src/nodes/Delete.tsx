import * as schema from '@stencila/schema'
import { InlineContentArray } from './InlineContent'

export function Delete(props: { node: schema.Delete }) {
  return (
    <del itemtype="http://schema.stenci.la/Delete">
      <InlineContentArray nodes = {props.node.content}></InlineContentArray>
    </del>
  )
}
