import { fromJSON, toJSON } from './prelude'

const addon = require('../index.node')

// Initialize this module
addon.pubsubInit()

/**
 * Subscribe to a topic.
 * 
 * Note: Currently we only allow for one subscriber per topic
 * so if you subscribe a second time to a topic the first
 * subscription will be cancelled.
 *
 * @param topic The topic to subscribe to e.g. `logging`
 */
export function subscribe(
  topic: string,
  callback: (topic: string, data: unknown) => unknown
): void {
  return addon.pubsubSubscribe(topic, (topic: string, json: string) =>
    callback(topic, fromJSON(json))
  )
}

/**
 * Unsubscribe from a topic
 *
 * @param topic The topic to unsubscribe from
 */
export function unsubscribe(topic: string): void {
  return addon.pubsubUnsubscribe(topic)
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
  return addon.pubsubPublish(topic, toJSON(data))
}
