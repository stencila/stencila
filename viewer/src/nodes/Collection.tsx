import * as schema from '@stencila/schema'
import { For } from 'solid-js'
import { CreativeWork } from './CreativeWork'

export function Collection(props: { node: schema.Collection }) {
  return (
    <div itemtype="https://schema.org/Collection">
      <For each={props.node.parts}>
        {(node) => <CreativeWork node={node}></CreativeWork>}
      </For>
    </div>
  )
}
