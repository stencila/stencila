import * as schema from '@stencila/schema'
import {
  createContext,
  createSignal,
  createState,
  Match,
  Switch,
} from 'solid-js'
import { CreativeWork } from './nodes/CreativeWork'

export const DocumentContext = createContext<schema.CreativeWork>()

export function DocumentProvider(props: { document: schema.CreativeWork }) {
  return (
    <DocumentContext.Provider value={props.document}>
      <CreativeWork node={props.document}></CreativeWork>
    </DocumentContext.Provider>
  )
}

export function DocumentSubscriber(props: { path?: string }) {
  const [document, setDocument] = createState(schema.creativeWork())
  const [ready, setReady] = createSignal(false)

  fetch(props.path ?? window.location.pathname, {
    headers: {
      accept: 'application/json',
    },
  })
    .then((response) => response.json())
    .then((data) => {
      setDocument(data)
      setReady(true)
    })

  return (
    <Switch>
      <Match when={!ready()}>
        <div>Loading...</div>
      </Match>
      <Match when={ready()}>
        <DocumentContext.Provider value={document}>
          <CreativeWork node={document}></CreativeWork>
        </DocumentContext.Provider>
      </Match>
    </Switch>
  )
}
