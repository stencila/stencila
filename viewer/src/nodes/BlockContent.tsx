import * as schema from '@stencila/schema'
import { For } from 'solid-js'
import { Dynamic } from 'solid-js/web'
import { CodeChunk } from './CodeChunk'
import { Collection } from './Collection'
import { Figure } from './Figure'
import { Heading } from './Heading'
import { Paragraph } from './Paragraph'
import { Table } from './Table'

export function BlockContent(props: { node: schema.BlockContent }) {
  const component = () => {
    switch (props.node.type) {
      case 'CodeChunk':
        return CodeChunk
      case 'Collection':
        return Collection
      case 'Heading':
        return Heading
      case 'Figure':
        return Figure
      case 'Paragraph':
        return Paragraph
      case 'Table':
        return Table
      default:
        return () => (
          <div class="unsupported">
            Unsupported block content type {schema.nodeType(props.node)}{' '}
          </div>
        )
    }
  }
  // @ts-ignore
  return <Dynamic component={component()} node={props.node}></Dynamic>
}

export function BlockContentArray(props: {
  nodes: schema.BlockContent[] | undefined
}) {
  return (
    props.nodes && (
      <For each={props.nodes}>
        {(node) => <BlockContent node={node}></BlockContent>}
      </For>
    )
  )
}
