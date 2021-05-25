import * as schema from '@stencila/schema'
import { InlineContentArray } from './InlineContent'

export function Link(props: { node: schema.Link }) {
  return (
    <a itemtype="http://schema.stenci.la/Link" href={props.node.target}>
      <InlineContentArray nodes = {props.node.content}></InlineContentArray>
    </a>
  )
}
