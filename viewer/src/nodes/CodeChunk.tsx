import * as schema from '@stencila/schema'
import { InlineContent } from '@stencila/schema'
import { ContentArray } from './Content'

export function CodeChunk(props: { node: schema.CodeChunk }) {
  return (
    <stencila-code-chunk
      itemtype="http://schema.stenci.la/CodeChunk"
      itemscope
      attr:data-programminglanguage={props.node.programmingLanguage}
    >
      <pre slot="text">
        <code>{props.node.text}</code>
      </pre>
      {props.node.outputs && (
        <figure slot="outputs">
          <ContentArray nodes={props.node.outputs as InlineContent[]} />
        </figure>
      )}
    </stencila-code-chunk>
  )
}
