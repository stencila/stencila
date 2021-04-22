import { init, test as testEvents } from './logging'
import { subscribe } from './subscriptions'

test('logging', async () => {
  // Initialize logging so it publishes events on the `logging` topic
  init()

  // Subscribe to topic, storing any events
  let events: unknown[] = []
  subscribe('logging', (_topic: string, data: unknown) => {
    events.push(data)
  })

  // Create some test logging events in Rust
  testEvents()

  // Wait a little until all events are published
  await new Promise((resolve) => setTimeout(resolve, 500))

  expect(events).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        message: 'A debug event',
        metadata: expect.objectContaining({
          fields: ['message'],
          file: 'rust/src/logging.rs',
          is_event: true,
          is_span: false,
          level: 'DEBUG',
          module_path: 'stencila::logging',
          target: 'stencila::logging',
        }),
      }),
      expect.objectContaining({
        message: 'An info event',
        metadata: expect.objectContaining({
          level: 'INFO',
        }),
      }),
      expect.objectContaining({
        message: 'A warn event',
        metadata: expect.objectContaining({
          level: 'WARN',
        }),
      }),
      expect.objectContaining({
        message: 'An error event',
        metadata: expect.objectContaining({
          level: 'ERROR',
        }),
      }),
    ])
  )
})
