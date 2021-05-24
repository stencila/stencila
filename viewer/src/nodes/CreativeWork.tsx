import * as schema from '@stencila/schema'
import { For, Match, Switch } from 'solid-js'
import { Article } from './Article'
import { Collection } from './Collection'
import { Figure } from './Figure'
import { Table } from './Table'

export function CreativeWork(props: { node: schema.CreativeWork }) {
  return (
    <Switch
      fallback={
        <span class="unsupported">
          Unsupported creative work type {schema.nodeType(props.node)}
        </span>
      }
    >
      <Match when={schema.isA('Article', props.node)}>
        <Article node={props.node as schema.Article} />
      </Match>
      <Match when={schema.isA('Collection', props.node)}>
        <Collection node={props.node as schema.Collection} />
      </Match>
      <Match when={schema.isA('Figure', props.node)}>
        <Figure node={props.node as schema.Figure} />
      </Match>
      <Match when={schema.isA('Table', props.node)}>
        <Table node={props.node as schema.Table} />
      </Match>
    </Switch>
  )
}

export function CreativeWorkReferences(props: {
  references: Exclude<schema.CreativeWork['references'], undefined>
}) {
  return (
    <>
      <h1>References</h1>
      <ol>
        <For each={props.references}>
          {(reference) => <li>{reference.toString()}</li>}
        </For>
      </ol>
    </>
  )
}
