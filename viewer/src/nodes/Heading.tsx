import * as schema from '@stencila/schema'
import { Match, Switch } from 'solid-js'
import { InlineContentArray } from './InlineContent'

export function Heading(props: { node: schema.Heading }) {
  const itemtype = 'https://schema.stenci.la/Heading'
  const depth = () => props.node.depth ?? 1
  const content = () => props.node.content

  return (
    <Switch
      fallback={
        <div class="unsupported">Unsupported heading depth {depth()} </div>
      }
    >
      <Match when={depth() == 1}>
        <h1 itemtype={itemtype} itemscope>
          <InlineContentArray nodes={content()}></InlineContentArray>
        </h1>
      </Match>
      <Match when={depth() == 2}>
        <h2 itemtype={itemtype} itemscope>
          <InlineContentArray nodes={content()}></InlineContentArray>
        </h2>
      </Match>
      <Match when={depth() == 3}>
        <h3 itemtype={itemtype} itemscope>
          <InlineContentArray nodes={content()}></InlineContentArray>
        </h3>
      </Match>
      <Match when={depth() == 4}>
        <h4 itemtype={itemtype} itemscope>
          <InlineContentArray nodes={content()}></InlineContentArray>
        </h4>
      </Match>
      <Match when={depth() == 5}>
        <h5 itemtype={itemtype} itemscope>
          <InlineContentArray nodes={content()}></InlineContentArray>
        </h5>
      </Match>
      <Match when={depth() == 6}>
        <h6 itemtype={itemtype} itemscope>
          <InlineContentArray nodes={content()}></InlineContentArray>
        </h6>
      </Match>
    </Switch>
  )
}
