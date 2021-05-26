import * as schema from '@stencila/schema'
import { Switch, Match, useContext } from 'solid-js'
import { InlineContentArray } from './InlineContent'
import { DocumentContext } from '../Document'

export function Cite(props: { node: schema.Cite }) {
  const content = () => props.node.content ?? []
  return (
    <cite itemtype="http://schema.stenci.la/Cite">
      <a href={'#' + props.node.target}>
        <Switch>
          <Match when={content().length > 0}>
            <InlineContentArray nodes={content()}></InlineContentArray>
          </Match>
          <Match when={content().length == 0}>
            <CiteContent node={props.node}></CiteContent>
          </Match>
        </Switch>
      </a>
    </cite>
  )
}

function CiteContent(props: { node: schema.Cite }) {
  const documentContext = useContext(DocumentContext)
  
  const content = () => {
    const references = documentContext?.root.references ?? []
    const target = props.node.target

    const reference = references.find((ref, index) =>
      typeof ref === 'string'
        ? `ref${index + 1}` === target ||
          `bib${index + 1}` === target
        : ref.id === target
    )
    
    if (reference === undefined) {
      return target
    } else {
      return <span>{(references?.indexOf(reference) ?? -2) + 1}</span>
    }
  }

  return content()
}
