import * as schema from '@stencila/schema'
import { BlockContentArray } from './BlockContent'
import { CreativeWorkReferences } from './CreativeWork'

export function Article(props: { node: schema.Article }) {
  return (
    <article itemtype="http://schema.org/Article" itemscope>
      {(props.node.content ?? []).length ? (
        <BlockContentArray nodes={props.node.content}></BlockContentArray>
      ) : (
        <div class="empty">Empty article</div>
      )}

      {props.node.references && (
        <CreativeWorkReferences
          references={props.node.references}
        ></CreativeWorkReferences>
      )}
    </article>
  )
}
