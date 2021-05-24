import * as schema from '@stencila/schema'
import { For, Match, Switch } from 'solid-js'
import { Cite } from './Cite'
import { CiteGroup } from './CiteGroup'
import { Delete } from './Delete'
import { Emphasis } from './Emphasis'
import { ImageObject } from './ImageObject'
import { Link } from './Link'
import { Strong } from './Strong'
import { Subscript } from './Subscript'
import { Superscript } from './Superscript'

export function InlineContent(props: { node: schema.InlineContent }) {
  return (
    <Switch
      fallback={
        <span class="unsupported">
          Unsupported inline content type {schema.nodeType(props.node)}
        </span>
      }
    >
      <Match when={schema.nodeType(props.node) === 'String'}>
        {props.node?.toString()}
      </Match>
      <Match when={schema.isA('Cite', props.node)}>
        <Cite node={props.node as schema.Cite}></Cite>
      </Match>
      <Match when={schema.isA('CiteGroup', props.node)}>
        <CiteGroup node={props.node as schema.CiteGroup}></CiteGroup>
      </Match>
      <Match when={schema.isA('Delete', props.node)}>
        <Delete node={props.node as schema.Delete}></Delete>
      </Match>
      <Match when={schema.isA('Emphasis', props.node)}>
        <Emphasis node={props.node as schema.Emphasis}></Emphasis>
      </Match>
      <Match when={schema.isA('ImageObject', props.node)}>
        <ImageObject node={props.node as schema.ImageObject}></ImageObject>
      </Match>
      <Match when={schema.isA('Link', props.node)}>
        <Link node={props.node as schema.Link}></Link>
      </Match>
      <Match when={schema.isA('Strong', props.node)}>
        <Strong node={props.node as schema.Strong}></Strong>
      </Match>
      <Match when={schema.isA('Subscript', props.node)}>
        <Subscript node={props.node as schema.Subscript}></Subscript>
      </Match>
      <Match when={schema.isA('Superscript', props.node)}>
        <Superscript node={props.node as schema.Superscript}></Superscript>
      </Match>
    </Switch>
  )
}

export function InlineContentArray(props: {
  nodes: schema.InlineContent[] | undefined
}) {
  return (
    props.nodes && (
      <For each={props.nodes}>
        {(node) => <InlineContent node={node}></InlineContent>}
      </For>
    )
  )
}
