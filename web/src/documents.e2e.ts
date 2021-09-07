import { Document } from 'stencila'
import { Client, connect, disconnect } from './client'
import {
  open,
  execute,
  change,
  subscribe,
  unsubscribe,
  DocumentEvent,
  close,
} from './documents'

jest.setTimeout(10000)

const clientId = 'cl-document-tests'
let client: Client
beforeAll(async () => {
  client = await connect(
    process.env.SERVER_URL || 'ws://127.0.0.1:9000/~ws',
    clientId
  )
})
afterAll(async () => {
  disconnect(client)
})

// Test of the basic document workflow of opening a document, subscribing
// to it, executing it, changing it, receiving events and then unsubscribing from it.
test('basic', async () => {
  let document: Document

  // Open a document
  document = await open(client, 'fixtures/articles/code.md')
  expect(document).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^do-[0-9a-zA-Z]{20}/),
      name: 'code.md',
    })
  )

  // Subscribe to updates to node values
  let events: DocumentEvent[] = []
  document = await subscribe(client, document.id, 'node:value', (event) => {
    expect(event.type).toBe('NodeValueUpdated')
    events.push(event)
  })
  expect(document).toEqual(
    expect.objectContaining({
      subscriptions: { 'node:value': [clientId] },
    })
  )

  // Send a change to the to the document
  document = await change(client, document.id, 'nodeId', {})

  // Unsubscribe from document
  document = await unsubscribe(client, document.id, 'node:value')
  expect(document).toEqual(
    expect.objectContaining({
      subscriptions: {},
    })
  )

  // Close the document
  await close(client, document.id)
})
