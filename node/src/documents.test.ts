import fs from 'fs'
import tmp from 'tmp'
import {
  close,
  get,
  list,
  load,
  open,
  subscribe,
  unsubscribe,
} from './documents'
import { DocumentEvent } from './types'

test.skip('workflow', async () => {
  const path = tmp.fileSync({ postfix: '.md' }).name
  fs.writeFileSync(path, '')

  let events: DocumentEvent[] = []

  // Open the document
  expect(open(path)).toEqual(
    expect.objectContaining({
      format: 'md',
      temporary: false,
      subscriptions: {},
    })
  )

  // Subscribe an editor panel to some of the document's topics
  subscribe(path, ['removed', 'renamed', 'modified'], (_topic, event) =>
    events.push(event)
  )
  expect(get(path).subscriptions).toEqual({
    removed: 1,
    renamed: 1,
    modified: 1,
  })

  // Subscribe a preview panel to the the `encoded:json` topic
  subscribe(path, ['encoded:json'], (_topic, event) => events.push(event))
  expect(get(path).subscriptions).toEqual({
    removed: 1,
    renamed: 1,
    modified: 1,
    'encoded:json': 1,
  })

  // Load some new content into the document (and wait a bit for events)
  load(path, 'Some content')
  await new Promise((resolve) => setTimeout(resolve, 300))
  expect(events).toEqual([
    expect.objectContaining({
      type: 'encoded',
      format: 'json',
    }),
  ])

  // Modify the file on disk (and wait a bit for events)
  events = []
  fs.writeFileSync(path, 'Some newer content that gets written to disk')
  await new Promise((resolve) => setTimeout(resolve, 3000))
  expect(events).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        type: 'modified',
        content: expect.stringMatching(/Some newer content/),
      }),
      expect.objectContaining({
        type: 'encoded',
        format: 'json',
      }),
    ])
  )

  // Unsubscribe from `encoded:json` because say we closed the preview panel
  unsubscribe(path, ['encoded:json'])
  expect(get(path).subscriptions).toEqual({
    removed: 1,
    renamed: 1,
    modified: 1,
  })

  // Close the document
  close(path)
  expect(list()).toEqual([])

  fs.unlinkSync(path)
})
