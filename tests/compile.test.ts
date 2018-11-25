import compile from '../src/compile'
import Thing from '../src/types/Thing'

test('compile:Thing', () => {
  const thing = new Thing()
  expect(compile('{"type": "Thing"}')).toEqual(thing)
  expect(compile({type: 'Thing'})).toEqual(thing)
  expect(compile(thing)).toEqual(thing)
})
