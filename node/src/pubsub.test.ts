import { subscribe, unsubscribe, publish } from './pubsub'

describe('subscriptions', () => {
  test('subscribe, publish, unsubscribe', async () => {
    let promise = new Promise((resolve) => {
      subscribe('topic', (topic, data) => {
        resolve([topic, data])
      })
    })

    publish('topic', 'data')

    let res = ((await promise) as unknown) as string[]
    expect(res[0]).toBe('topic')
    expect(res[1]).toBe('data')

    unsubscribe('topic')
  })

  test('publish with no subscribers', async () => {
    publish('topic', 'data')
  })

  test('unsubscribe without subscribing', async () => {
    unsubscribe('topic')
  })
})
