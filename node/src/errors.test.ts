import { open } from './documents'
import { Error } from './types'

test('workflow', async () => {
  // Open a document that does not exist and parse the error
  // message as a JSON object.
  let error
  try {
    open('foo.bar')
  } catch (err) {
    error = JSON.parse(err.message)
  }
  expect(error).toEqual(
    expect.objectContaining({
      type: 'Unknown',
      message: expect.stringContaining('No such file or directory'),
    })
  )
})
