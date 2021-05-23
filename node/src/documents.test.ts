import fs from 'fs'
import tmp from 'tmp'
import {
  close,
  get,
  list,
  load,
  open,
  subscribe,
  unsubscribe
} from './documents'
import { DocumentEvent } from './types'

test('workflow', async () => {
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

  // Subscribe a preview panel to the the `converted:html` topic
  subscribe(path, ['converted:html'], (_topic, event) => events.push(event))
  expect(get(path).subscriptions).toEqual({
    removed: 1,
    renamed: 1,
    modified: 1,
    'converted:html': 1,
  })

  // Load some new content into the document (and wait a bit for events)
  load(path, 'Some content')
  await new Promise((resolve) => setTimeout(resolve, 300))
  expect(events).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        type: 'converted',
        format: 'html',
        content: expect.stringMatching(/TODO: Convert to html/),
      }),
    ])
  )

  // Modify the file on disk (and wait a bit for events)
  fs.writeFileSync(path, 'Some newer content that gets written to disk')
  await new Promise((resolve) => setTimeout(resolve, 600))
  expect(events).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        type: 'modified',
        content: expect.stringMatching(/Some newer content/),
      }),
      expect.objectContaining({
        type: 'converted',
        format: 'html',
      }),
    ])
  )

  // Unsubscribe from `converted:html` because say we closed the preview panel
  unsubscribe(path, ['converted:html'])
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
