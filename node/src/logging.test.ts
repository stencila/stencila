import { init, test as testEvents } from './logging'
import { subscribe } from './pubsub'

test('logging', async () => {
  // Initialize logging so it publishes events on the `logging` topic
  // down to debug level
  init({
    logging: {
      desktop: {
        level: 'debug',
      },
    },
  })

  // Subscribe to topic, storing any events
  let events: unknown[] = []
  subscribe('logging', (_topic: string, data: unknown) => {
    events.push(data)
  })

  // Create some test logging events in Rust
  testEvents()

  // Wait a little until all events are published
  await new Promise((resolve) => setTimeout(resolve, 300))

  expect(events).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        message: 'A debug event',
        metadata: expect.objectContaining({
          level: 'DEBUG',
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
