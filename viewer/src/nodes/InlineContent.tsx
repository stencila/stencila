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
  const component = () => {
    switch (schema.nodeType(props.node)) {
      case 'Null':
      case 'Boolean':
      case 'Number':
      case 'String':
        return () => props.node
      case 'Cite':
        return Cite
      case 'CiteGroup':
        return CiteGroup
      case 'Delete':
        return Delete
      case 'Emphasis':
        return Emphasis
      case 'ImageObject':
        return ImageObject
      case 'Link':
        return Link
      case 'Strong':
        return Strong
      case 'Subscript':
        return Subscript
      case 'Superscript':
        return Superscript
      default:
        return () => (
          <div class="unsupported">
            Unsupported inline content type {schema.nodeType(props.node)}{' '}
          </div>
        )
    }
  }
  // @ts-ignore
  return <Dynamic component={component()} node={props.node}></Dynamic>
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
