import { test as testEvents } from './logging'
import { subscribe } from './pubsub'

test('logging', async () => {
  // Subscribe to topic, storing any events
  // Note: currently this will override any other subscriptions
  // to the logging topic.
  let events: unknown[] = []
  subscribe('logging', (_topic: string, data: unknown) => {
    events.push(data)
  })

  // Create some test logging events in Rust
  testEvents()

  // Wait a little until all events are published
  await new Promise((resolve) => setTimeout(resolve, 300))

  // Don't expect to get the DEBUG event unless the desktop
  // logging config says so
  expect(events).toEqual(
    expect.arrayContaining([
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
