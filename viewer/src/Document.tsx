import * as schema from '@stencila/schema'
import {
  createContext,
  createSignal,
  createState,
  Match,
  Switch,
} from 'solid-js'
import { CreativeWork } from './nodes/CreativeWork'

/**
 * The document context that is implicitly passed down
 * the component tree.
 *
 * Can be used by components to alter rendering based on
 * properties of the current document.
 */
interface DocumentContext {
  /**
   * The URL of the current document.
   *
   * Used to resolve the URLs of media e.g. `ImageObject`.
   */
  url?: string

  /**
   * The root node of the current document.
   *
   * Used for things like creating the content of `Cite` nodes
   * by accessing the document's `references`.
   */
  root: schema.CreativeWork
}

export const DocumentContext = createContext<DocumentContext>()

/**
 * A `DocumentContext` provider intended for rendering a
 * document to a string (i.e. server-side rendering).
 *
 * @param props.document The `CreativeWork` to be rendered.
 */
export function DocumentRenderer(props: { document: schema.CreativeWork }) {
  // TODO use `solid-meta` `Title`, `Meta` for keywords and other document metadata
  return (
    <DocumentContext.Provider value={{ root: props.document }}>
      <CreativeWork node={props.document}></CreativeWork>
    </DocumentContext.Provider>
  )
}

/**
 * A `DocumentContext` provider, intended mainly for testing,
 * that fetches a document from a URL.
 *
 * @param props.url The URL of the JSON representation of the
 *                  `CreativeWork` to be rendered. Defaults to
 *                  the path of the current window.
 */
export function DocumentFetcher(props: { url?: string }) {
  const [document, setDocument] = createState(schema.creativeWork())
  const [ready, setReady] = createSignal(false)

  let url = props.url
  if (url === undefined) {
    // Fallback to using URL parameter
    const params = new URLSearchParams(window.location.search)
    url = params.get('url') ?? undefined
  }
  if (url === undefined) {
    // Fallback to getting URL from current path
    const path = window.location.pathname.slice(1)
    if (path.startsWith('http://') || path.startsWith('https://')) {
      // Remote URL, use as is
      url = path
    } else {
      // Local path, so need to reinstate the leading slash
      url = '/' + path
    }
  }

  if (url !== '') {
    // Fetch the document from the url
    fetch(url, {
      headers: {
        accept: 'application/json',
      },
    })
      .then((response) => response.json())
      .then((document) => {
        setDocument(document)
        setReady(true)
      })
  } else {
    // Create an empty article
    setDocument(schema.article())
    setReady(true)
  }

  return (
    <Switch>
      <Match when={!ready()}>
        <div>Loading...</div>
      </Match>
      <Match when={ready()}>
        <DocumentContext.Provider value={{ url, root: document }}>
          <CreativeWork node={document}></CreativeWork>
        </DocumentContext.Provider>
      </Match>
    </Switch>
  )
}

/**
 * A `DocumentContext` provider that subscribes to a `CreativeWork`
 * and updates it's local state in response to published changes.
 *
 * @param props.url The URL of the JSON representation of the
 *                  `CreativeWork` to be rendered. Defaults to
 *                  the path of the current window.
 */
export function DocumentSubscriber(props: { url?: string }) {
  const [document, setDocument] = createState(schema.creativeWork())
  const [ready, setReady] = createSignal(false)

  // TODO Set up websocket connection

  return (
    <Switch>
      <Match when={!ready()}>
        <div>Loading...</div>
      </Match>
      <Match when={ready()}>
        <DocumentContext.Provider value={{ root: document }}>
          <CreativeWork node={document}></CreativeWork>
        </DocumentContext.Provider>
      </Match>
    </Switch>
  )
}
