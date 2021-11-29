/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { Document, DocumentEvent } from '@stencila/stencila'
import { Client, connect, disconnect } from './client'
import { close, open, subscribe, unsubscribe } from './documents'

jest.setTimeout(10000)

const clientId = 'cl-document-tests'
let client: Client
beforeAll(async () => {
  client = await connect(
    'pr-document-tests',
    clientId,
    process.env.SERVER_URL ?? 'ws://127.0.0.1:9000'
  )
})
afterAll(() => {
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
  const events: DocumentEvent[] = []
  document = await subscribe(client, document.id, 'patched', (event) => {
    expect(event.type).toBe('patched')
    events.push(event)
  })
  expect(document).toEqual(
    expect.objectContaining({
      subscriptions: { patched: [clientId] },
    })
  )

  // Unsubscribe from document
  document = await unsubscribe(client, document.id, 'patched')
  expect(document).toEqual(
    expect.objectContaining({
      subscriptions: {},
    })
  )

  // Close the document
  await close(client, document.id)
})
