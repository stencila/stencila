import * as schema from '@stencila/schema'
import { BlockContentArray } from './BlockContent'
import { ContentArray } from './Content'

export function Figure(props: { node: schema.Figure }) {
  return (
    <figure itemtype="http://schema.stenci.la/Figure" itemscope id={props.node.id}>
      {props.node.label && <label data-itemprop="label">{props.node.label}</label>}
      {props.node.content && (
        <ContentArray
          nodes={props.node.content as (schema.InlineContent | schema.BlockContent)[]}
        ></ContentArray>
      )}
      {props.node.caption && (
        <figcaption>
          {typeof props.node.caption === 'string' ? (
            props.node.caption
          ) : (
            <BlockContentArray nodes={props.node.caption}></BlockContentArray>
          )}
        </figcaption>
      )}
    </figure>
  )
}
