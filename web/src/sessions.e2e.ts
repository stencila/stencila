import { connect, disconnect } from './client'
import { start, stop, subscribe, unsubscribe } from './sessions'
import { Session, SessionEvent } from './types'

jest.setTimeout(10000)

// Test of the basic session workflow of starting a session, subscribing
// to it, receiving events and then stopping it.
test('basic', async () => {
  let client = await connect()
  let session: Session

  // Start the session
  session = await start(client, 'projectId', 'snapshotId')
  expect(session).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^se-[0-9a-zA-Z]{20}/),
      project: 'projectId',
      snapshot: 'snapshotId',
      status: 'Started',
    })
  )

  // Subscribe to updates
  let updates: SessionEvent[] = []
  session = await subscribe(client, session.id, 'updated', (event) => {
    expect(event.type).toBe('Updated')
    updates.push(event)
  })
  expect(session).toEqual(
    expect.objectContaining({
      subscriptions: { updated: ['clientId'] },
    })
  )

  // Subscribe to heartbeats
  let heartbeats: SessionEvent[] = []
  session = await subscribe(client, session.id, 'heartbeat', (event) => {
    expect(event.type).toBe('Heartbeat')
    heartbeats.push(event)
  })
  expect(session).toEqual(
    expect.objectContaining({
      subscriptions: { updated: ['clientId'], heartbeat: ['clientId'] },
    })
  )

  // The above subscription should fire an `updated` event
  expect(updates).toEqual([
    expect.objectContaining({
      type: 'Updated',
      session: expect.objectContaining({
        id: session.id,
      }),
    }),
  ])

  // Wait for a heartbeat. If this isn't working, this test should timeout
  await new Promise<void>((resolve) => {
    setInterval(() => {
      if (heartbeats.length > 0) {
        return resolve()
      }
    }, 1000)
  })
  expect(heartbeats).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        type: 'Heartbeat',
        session: expect.objectContaining({
          id: session.id,
        }),
      }),
    ])
  )

  // Unsubscribe from heartbeats; should still be subscribed to updates
  session = await unsubscribe(client, session.id, 'heartbeat')
  expect(session).toEqual(
    expect.objectContaining({
      subscriptions: { updated: ['clientId'] },
    })
  )

  // Unsubscribe from updates
  session = await unsubscribe(client, session.id, 'updated')
  expect(session).toEqual(
    expect.objectContaining({
      subscriptions: {},
    })
  )

  // Stop the session
  session = await stop(client, session.id)
  expect(session).toEqual(
    expect.objectContaining({
      status: 'Stopped',
    })
  )

  disconnect(client)
})
