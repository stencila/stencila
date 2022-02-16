import { Session, SessionEvent } from '@stencila/stencila'
import { Client } from './client'
import { ProjectId } from './types'

export type SessionId = string

type SessionTopic = 'updated' | 'heartbeat'

/**
 * Start a session
 */
export async function start(
  client: Client,
  projectId: ProjectId
): Promise<Session> {
  return client.call('sessions.start', {
    projectId,
  }) as Promise<Session>
}

/**
 * Stop a session
 */
export async function stop(
  client: Client,
  sessionId: SessionId
): Promise<Session> {
  return client.call('sessions.stop', {
    sessionId,
  }) as Promise<Session>
}

/**
 * Default handler for session events
 *
 * Dispatches a `CustomEvent` with the type of the event,
 * prefixed with "session:" e.g. "session:heartbeat".
 */
function defaultHandler(event: SessionEvent): void {
  window.dispatchEvent(
    new CustomEvent(`session:${event.type}`.toLowerCase(), { detail: event })
  )
}

/**
 * Subscribe to a session topic
 *
 * Note that it is possible have a single handler which handles events for multiple
 * topics (i.e. call this function multiple times with the same handler).
 * In general, to reduce noise, only subscribe to the topics for which events are actually
 * being used e.g. to update an UI indicator of the state of the session.
 * Given that, it is probably better to have one handler function per topic.
 *
 * Returns the session state after the topic has been subscribed to.
 */
export async function subscribe(
  client: Client,
  sessionId: SessionId,
  topic: SessionTopic,
  handler: (event: SessionEvent) => void = defaultHandler
): Promise<Session> {
  client.on(`sessions:${sessionId}:${topic}`, handler)
  return client.call('sessions.subscribe', {
    sessionId,
    topic,
  }) as Promise<Session>
}

/**
 * Unsubscribe from a session topic
 *
 * Returns the session state after the topic has been unsubscribed from.
 */
export async function unsubscribe(
  client: Client,
  sessionId: SessionId,
  topic: SessionTopic
): Promise<Session> {
  client.off(`sessions:${sessionId}:${topic}`)
  return client.call('sessions.unsubscribe', {
    sessionId,
    topic,
  }) as Promise<Session>
}
