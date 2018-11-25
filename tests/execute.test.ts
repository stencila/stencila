import execute from '../src/execute'
import Thing from '../src/Thing'

test('execute:Thing', () => {
  const thing = new Thing()
  expect(execute('{"type": "Thing"}')).toEqual(thing)
  expect(execute({type: 'Thing'})).toEqual(thing)
  expect(execute(thing)).toEqual(thing)
})
