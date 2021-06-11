import { open } from './documents'
import path from 'path'
import { subscribe, unsubscribe } from './errors'
import { Error } from './types'

test('workflow', async () => {
  let lastError: Error | undefined = undefined

  // Subscribe to error events
  subscribe((_topic, error) => (lastError = error as Error))

  // Open a document that does not exist
  expect(() => open('foo.bar')).toThrow(/^No such file or directory/)
  await new Promise((resolve) => setTimeout(resolve, 100))
  expect(lastError).toEqual(
    expect.objectContaining({
      type: 'Unknown',
      message: expect.stringContaining('No such file or directory'),
    })
  )

  /*
  TODO: This no longer throws this error
  // Open a document that does exist but specify a format which
  // no plugins can handle
  const readme = path.join(__dirname, '..', 'README.md')
  expect(() => open(readme, 'foo')).toThrow(
    /^None of the registered plugins implement method 'decode' with given parameters/
  )
  await new Promise((resolve) => setTimeout(resolve, 100))
  expect(lastError).toEqual(
    expect.objectContaining({
      type: 'UndelegatableCall',
      message: expect.stringContaining(
        'None of the registered plugins implement'
      ),
      method: 'decode',
      params: expect.objectContaining({ format: 'foo' }),
    })
  )
  */

  // Unsubscribe from error events
  unsubscribe()
})
