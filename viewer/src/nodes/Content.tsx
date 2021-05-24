import * as schema from '@stencila/schema'
import { For, Match, Switch } from 'solid-js'
import { BlockContent } from './BlockContent'
import { InlineContent } from './InlineContent'

export function Content(props: {
  node: schema.InlineContent | schema.BlockContent
}) {
  return (
    <Switch
      fallback={
        <div class="unsupported">
          Unsupported content type {schema.nodeType(props.node)}{' '}
        </div>
      }
    >
      <Match when={schema.isInlineContent(props.node)}>
        <InlineContent node={props.node as schema.InlineContent} />
      </Match>
      <Match when={schema.isBlockContent(props.node)}>
        <BlockContent node={props.node as schema.BlockContent} />
      </Match>
    </Switch>
  )
}

export function ContentArray(props: {
  nodes: (schema.BlockContent | schema.BlockContent)[] | undefined
}) {
  return (
    props.nodes && (
      <For each={props.nodes}>
        {(node) => <Content node={node}></Content>}
      </For>
    )
  )
}
