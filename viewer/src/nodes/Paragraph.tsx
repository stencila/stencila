import * as schema from '@stencila/schema'
import { InlineContentArray } from './InlineContent'

export function Paragraph(props: { node: schema.Paragraph }) {
  return (
    <p itemtype="https://schema.stenci.la/Paragraph">
      <InlineContentArray nodes = {props.node.content}></InlineContentArray>
    </p>
  )
}
