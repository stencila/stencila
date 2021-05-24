import * as schema from '@stencila/schema'
import { For } from 'solid-js'
import { Cite } from './Cite'

export function CiteGroup(props: { node: schema.CiteGroup }) {
  return (
    <span itemtype="http://schema.stenci.la/CiteGroup">
      <For each={props.node.items}>{(node) => <Cite node={node}></Cite>}</For>
    </span>
  )
}
