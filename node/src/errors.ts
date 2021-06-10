import { fromJSON } from './prelude'
import { Subscriber } from './pubsub'

const addon = require('../index.node')

/**
 * Subscribe to the "errors" topic.
 */
export function subscribe(subscriber: Subscriber): void {
  return addon.pubsubSubscribe('errors', (topic: string, json: string) =>
    subscriber(topic, fromJSON(json))
  )
}

/**
 * Unsubscribe from the "error" topic
 */
export function unsubscribe(): void {
  return addon.pubsubUnsubscribe('errors')
}
