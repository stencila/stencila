/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { article, creativeWork } from '@stencila/schema'
import { Document, DocumentEvent } from '../types'
import { Client, connect, disconnect } from './client'
import {
  close,
  create,
  dump,
  load,
  open,
  subscribe,
  unsubscribe,
} from './documents'

jest.setTimeout(60000)

const clientId = 'cl-document-tests'
let client: Client
beforeAll(async () => {
  client = await connect(
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

test('create', async () => {
  let document = await create(client)
  let json = await dump(client, document.id, 'json')
  expect(json).toMatch(/^{"type":"Article"/)

  document = await create(client, creativeWork({ content: ['Beep!'] }))
  const md = await dump(client, document.id, 'md')
  expect(md).toMatch(/^Beep!/)

  document = await create(client, 'Boop!', 'md')
  json = await dump(client, document.id, 'json')
  expect(json).toMatch(/"content":\["Boop!"\]/)
})

test('load', async () => {
  const document = await create(client)

  let ok = await load(client, document.id, article({ content: [] }))
  expect(ok).toBeTruthy()
  let json = await dump(client, document.id, 'json')
  expect(json).toMatch(/^{"type":"Article"/)

  ok = await load(client, document.id, 'Hello *world*!', 'md')
  expect(ok).toBeTruthy()
  json = await dump(client, document.id, 'html')
  expect(json).toContain(
    '<em itemtype="https://schema.stenci.la/Emphasis" itemscope><span>world</span>'
  )
})

test('dump', async () => {
  const document = await open(client, 'fixtures/nodes/creative-work.json')

  const json = await dump(client, document.id, 'json')
  expect(json).toMatch(/^{"type":"CreativeWork"/)

  const md = await dump(client, document.id, 'md')
  expect(md).toMatch(/^A fixture that is a creative work/)

  const rpng = await dump(client, document.id, 'rpng', 'cc-1')
  expect(rpng).toMatch(/^data:image\/png/)
})
