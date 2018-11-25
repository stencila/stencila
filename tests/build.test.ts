import build from '../src/build'
import Thing from '../src/types/Thing'

test('build:Thing', () => {
  const thing = new Thing()
  expect(build('{"type": "Thing"}')).toEqual(thing)
  expect(build({type: 'Thing'})).toEqual(thing)
  expect(build(thing)).toEqual(thing)
})
