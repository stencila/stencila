import { fromJSON, toJSON } from './prelude'

const addon = require('../index.node')

/**
 * Subscribe to a topic
 *
 * @param topic The topic to subscribe to e.g. `logging`
 */
export function subscribe(
  topic: string,
  callback: (topic: string, data: unknown) => unknown
): void {
  return addon.subscribe(topic, (topic: string, json: string) =>
    callback(topic, fromJSON(json))
  )
}

/**
 * Unsubscribe from a topic
 *
 * @param topic The topic to unsubscribe from
 */
export function unsubscribe(topic: string): void {
  return addon.unsubscribe(topic)
}

/**
 * Publish data for a topic
 *
 * This will not normally need to be called by Node.js but
 * is provided mainly for testing purposes.
 *
 * @param topic The topic to publish data for
 */
export function publish(topic: string, data: unknown): void {
  return addon.publish(topic, toJSON(data))
}
