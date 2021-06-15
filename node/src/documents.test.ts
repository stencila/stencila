import fs from 'fs'
import tmp from 'tmp'
import {
  close,
  create,
  get,
  load,
  open,
  subscribe,
  unsubscribe,
  write,
} from './documents'
import { DocumentEvent } from './types'

test('create', async () => {
  expect(create()).toEqual(
    expect.objectContaining({
      temporary: true,
      name: 'Unnamed',
    })
  )

  expect(create('md')).toEqual(
    expect.objectContaining({
      temporary: true,
      name: 'Unnamed',
      format: expect.objectContaining({
        name: 'md',
        binary: false,
        type: 'Article',
      }),
    })
  )
})

/**
 * Test of a workflow involving opening and modifying a document
 *
 * Uses a JSON document as input so that this test is not dependant
 * on having a converter plugin installed.
 */
test('workflow-open-modify', async () => {
  const path = tmp.fileSync({ postfix: '.json' }).name
  fs.writeFileSync(path, '{"type": "Article"}')

  let events: DocumentEvent[] = []

  // Open the document
  const doc = open(path)
  const docId = doc.id
  expect(doc).toEqual(
    expect.objectContaining({
      format: expect.objectContaining({
        name: 'json',
        binary: false,
      }),
      temporary: false,
      subscriptions: {},
    })
  )

  // Subscribe an editor panel to some of the document's topics
  subscribe(docId, ['removed', 'renamed', 'modified'], (_topic, event) =>
    events.push(event)
  )
  expect(get(docId).subscriptions).toEqual({
    removed: 1,
    renamed: 1,
    modified: 1,
  })

  // Subscribe a preview panel to the the `encoded:json` topic
  subscribe(docId, ['encoded:json'], (_topic, event) => events.push(event))
  expect(get(docId).subscriptions).toEqual({
    removed: 1,
    renamed: 1,
    modified: 1,
    'encoded:json': 1,
  })

  // Load some new content into the document (and wait a bit for events)
  load(
    docId,
    `{
    "type": "Article",
    "content": [{
      "type": "Paragraph",
      "content": ["Some content"]
    }]
  }`
  )
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(events).toEqual([
    expect.objectContaining({
      type: 'encoded',
      format: expect.objectContaining({
        name: "json",
        binary: false
      }),
    }),
  ])

  // Modify the file on disk (and wait a bit for events)
  events = []
  fs.writeFileSync(
    path,
    `{
    "type": "Article",
    "content": [{
      "type": "Paragraph",
      "content": ["Some new content"]
    }]
  }`
  )
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(events).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        type: 'modified',
        content: expect.stringMatching(/Some new content/),
      }),
      expect.objectContaining({
        type: 'encoded',
        format: expect.objectContaining({
          name: "json",
          binary: false
        }),
      }),
    ])
  )

  // Write the document from here (note there should be no `modified` event)
  events = []
  write(
    docId,
    `{
    "type": "Article",
    "content": [{
      "type": "Paragraph",
      "content": ["Some newer content"]
    }]
  }`
  )
  await new Promise((resolve) => setTimeout(resolve, 500))
  expect(events).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        type: 'encoded',
        format: expect.objectContaining({
          name: "json",
          binary: false
        }),
      }),
    ])
  )

  // Unsubscribe from `encoded:json` because say we closed the preview panel
  unsubscribe(docId, ['encoded:json'])
  expect(get(docId).subscriptions).toEqual({
    removed: 1,
    renamed: 1,
    modified: 1,
  })

  // Close the document
  close(docId)

  fs.unlinkSync(path)
})
