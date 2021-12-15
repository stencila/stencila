import { ViewUpdate } from '@codemirror/view'
import { ValidatorTypes } from '@stencila/schema'
import { Document, DocumentEvent, Operation, Patch } from '@stencila/stencila'
import { Client, ClientId } from './client'
import * as codemirror from './patches/codemirror'
import { applyPatch } from './patches/dom'

/**
 * The path of a document
 */
export type DocumentPath = string

/**
 * The id of a document
 */
export type DocumentId = string

/**
 * The id of a node within a document
 */
export type NodeId = string

/**
 * Possible document subscription topics
 */
type DocumentTopic = 'patched'

/**
 * The browser event emitted when a document is patched (e.g. by the
 * WYSIWYG article editor)
 */
export interface DocumentPatchEvent extends CustomEvent {
  detail: Patch
}

/**
 * The browser event emitted when the language of a text editor
 * is changed
 */
export interface LanguageChangeEvent extends CustomEvent {
  detail: {
    ext: string
    name: string
  }
}

/**
 * The browser event emitted when the content of a text editor
 * is changed
 */
export interface ContentChangeEvent extends CustomEvent {
  detail: ViewUpdate
}

/**
 * Open a document
 *
 * Loads the document into memory on the server and returns a document object
 * with an `id` which can be used to subscribe to topics.
 */
export async function open(
  client: Client,
  documentPath: DocumentPath
): Promise<Document> {
  return client.call('documents.open', {
    path: documentPath,
  }) as Promise<Document>
}

/**
 * Close a document
 */
export async function close(
  client: Client,
  documentId: DocumentId
): Promise<Document> {
  return client.call('documents.close', {
    documentId,
  }) as Promise<Document>
}

/**
 * Subscribe to a document topic
 */
export async function subscribe(
  client: Client,
  documentId: DocumentId,
  topic: DocumentTopic,
  handler: (event: DocumentEvent) => void
): Promise<Document> {
  client.on(`documents:${documentId}:${topic}`, handler)
  return client.call('documents.subscribe', {
    documentId,
    topic,
  }) as Promise<Document>
}

/**
 * Unsubscribe from a document topic
 */
export async function unsubscribe(
  client: Client,
  documentId: DocumentId,
  topic: DocumentTopic
): Promise<Document> {
  client.off(`documents:${documentId}:${topic}`)
  return client.call('documents.unsubscribe', {
    documentId,
    topic,
  }) as Promise<Document>
}

/**
 * Send a document patch to the server
 *
 * Will generate an error if the patch could not be
 * applied e.g. no node with the id could be found or
 * the patch was inconsistent with the node.
 */
export async function sendPatch(
  client: Client,
  documentId: DocumentId,
  patch: Patch
): Promise<void> {
  // During development it's very useful to see the patch operations being sent
  if (process.env.NODE_ENV !== 'production') {
    const { actor, target, ops } = patch
    console.log('📢 Sending patch:', JSON.stringify({ actor, target }))
    for (const op of ops) console.log('  ', JSON.stringify(op))
  }

  return client.call('documents.patch', {
    documentId,
    patch,
  }) as Promise<void>
}

// Receive a document patch from the server
//
// Handles a 'patched' event by either sending it to the relevant WebComponent
// so that it can make the necessary changes to the DOM, or by calling `applyPatch` which
// makes changes to the DOM directly.
export function receivePatch(clientId: ClientId, event: DocumentEvent): void {
  let patch
  if (event.type === 'patched') {
    patch = event.patch as Patch
  } else {
    console.error(
      `Expected document event to be of type 'patched', got type '${event.type}'`
    )
    return
  }

  const { actor, target, ops } = patch

  // Ignore any patches where this client was the actor
  if (actor === clientId) return

  // During development it's useful to see which patches are being received
  if (process.env.NODE_ENV !== 'production') {
    console.log('📩 Received patch:', JSON.stringify({ actor, target }))
    for (const op of ops) console.log('  ', JSON.stringify(op))
  }

  applyPatch(patch)
}

/**
 * Execute a document node
 *
 * Optionally, pass a patch to apply to the node
 * prior to executing it.
 */
export async function execute(
  client: Client,
  documentId: DocumentId,
  nodeId: NodeId,
  patch?: Patch
): Promise<void> {
  return client.call('documents.execute', {
    documentId,
    nodeId,
    patch,
  }) as Promise<void>
}

/**
 * Listen to browser window events that require passing on to server
 */
export function listen(
  client: Client,
  clientId: ClientId,
  documentId: DocumentId
): void {
  window.addEventListener('stencila-document-patch', (event) =>
    onDocumentPatch(client, clientId, documentId, event as DocumentPatchEvent)
  )
  window.addEventListener('stencila-language-change', (event) =>
    onLanguageChange(client, clientId, documentId, event as LanguageChangeEvent)
  )
  window.addEventListener('stencila-content-change', (event) =>
    onContentChange(client, clientId, documentId, event as ContentChangeEvent)
  )

  window.addEventListener('stencila-validator-change', (event) =>
    onValidatorChange(client, clientId, documentId, event as ContentChangeEvent)
  )

  window.addEventListener('stencila-parameter-change', (event) =>
    onParameterChange(client, clientId, documentId, event as ContentChangeEvent)
  )
}

/**
 * Handle a `DocumentPatchEvent`
 *
 * These events, created by document editors, are fully formed `Patch`es missing only the actor.
 */
async function onDocumentPatch(
  client: Client,
  clientId: ClientId,
  documentId: DocumentId,
  event: DocumentPatchEvent
): Promise<void> {
  return sendPatch(client, documentId, {
    actor: clientId,
    ...event.detail,
  })
}

/**
 * Handle a `LanguageChangeEvent`
 *
 * These events, created by text editors for individual nodes, need to be
 * transformed into a `Patch` targeting that node. The `slot` (the name of the property)
 * in the address is dependent upon the type of node
 */
async function onLanguageChange(
  client: Client,
  clientId: ClientId,
  documentId: DocumentId,
  event: LanguageChangeEvent
): Promise<void> {
  const [nodeType, nodeId] = resolveEventNode(event)
  const slot = nodeType.startsWith('Code')
    ? 'programmingLanguage'
    : 'mathLanguage'
  const value = (event.detail.ext ?? event.detail.name).toLowerCase()

  return sendPatch(client, documentId, {
    actor: clientId,
    target: nodeId,
    ops: [
      {
        type: 'Replace',
        address: [slot],
        value,
        items: 1,
        length: value.length,
      },
    ],
  })
}

/**
 * Handle a `ContentChangeEvent`
 *
 * These events, created by text editors for individual nodes, need to be
 * transformed into a `Patch` targeting that node.
 */
async function onContentChange(
  client: Client,
  clientId: ClientId,
  documentId: DocumentId,
  event: ContentChangeEvent
): Promise<void> {
  const [_nodeType, nodeId] = resolveEventNode(event)
  const slot = 'text'

  const update = event.detail
  if (update.docChanged) {
    const ops = codemirror.stateToOps(update.state, [slot])
    // const ops = codemirror.diffToOps(update.startState, update.state, [slot])
    const patch = {
      actor: clientId,
      target: nodeId,
      ops,
    }
    return sendPatch(client, documentId, patch)
  }
}

/**
 * The browser event emitted when either the type or property of a parameter validator changes.
 */
export interface ValidatorChangeEvent extends CustomEvent {
  detail:
    | {
        type: 'property'
        name: string
        value: string
      }
    | {
        type: 'validator'
        value: Exclude<ValidatorTypes['type'], 'Validator'>
      }
}

/**
 * Handle a `ValidatorChangeEvent`
 *
 * These events, created by text editors for individual nodes, need to be
 * transformed into a `Patch` targeting that node.
 */
async function onValidatorChange(
  client: Client,
  clientId: ClientId,
  documentId: DocumentId,
  event: ValidatorChangeEvent
): Promise<void> {
  console.log('validator changed:', event)
  const [_nodeType, nodeId] = resolveEventNode(event)

  const ops: Operation[] = []

  if (event.detail.type === 'property') {
    ops.push({
      type: 'Replace',
      address: ['validator', event.detail.name],
      value: event.detail.value,
      items: 1,
      length: 1,
    })
  } else {
    // TODO: Handle changing of validator type
  }

  const patch = {
    actor: clientId,
    target: nodeId,
    ops,
  }
  return sendPatch(client, documentId, patch)
}

/**
 * The browser event emitted when either the name of value of the parameter changes.
 */
export interface ParameterChangeEvent extends CustomEvent {
  detail: {
    property: 'name' | 'value'
    value: string
  }
}

/**
 * Handle a `ParameterChangeEvent`
 *
 * These events, created by text editors for individual nodes, need to be
 * transformed into a `Patch` targeting that node.
 */
async function onParameterChange(
  client: Client,
  clientId: ClientId,
  documentId: DocumentId,
  event: ParameterChangeEvent
): Promise<void> {
  console.log('parameter changed:', event)
  // TODO: Handle parameter change event
  const [_nodeType, nodeId] = resolveEventNode(event)

  const ops: Operation[] = [
    {
      type: 'Replace',
      address: [event.detail.property],
      value: event.detail.value,
      items: 1,
      length: 1,
    },
  ]

  const patch = {
    actor: clientId,
    target: nodeId,
    ops,
  }
  return sendPatch(client, documentId, patch)
}

/**
 * Extract the given Element's Schema Node type
 * If the node does not have an `itemtype` attribute, this function returns an empty string.
 */
const getElType = (targetEl: Element): string => {
  const itemtype = targetEl?.getAttribute('itemtype') ?? ''
  const parts = itemtype.split('/')
  return parts[parts.length - 1] ?? ''
}

/**
 * Get the type and id of the Stencila document node that the event
 * was emitted from.
 *
 * This is necessary to be able to determine the shape and target of the generated patch.
 */
function resolveEventNode(event: Event): [string, string] {
  const elem: HTMLElement | null = event.target as HTMLElement
  let id = elem.getAttribute('id')
  let elType = getElType(elem)

  if (id === null || id === '') {
    const nodeEl = elem.closest('[itemtype]')
    if (nodeEl) {
      id = nodeEl.getAttribute('id')
      elType = getElType(nodeEl)
    }
  }

  if (id !== null) {
    return [elType, id]
  }

  throw new Error('Unable to resolve the node which emitted the event')
}
