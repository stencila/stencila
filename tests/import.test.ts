import {default as import_, importJsonLd} from '../src/import'
import Thing from '../src/Thing'
import Person from '../src/Person'

test('import:Thing', () => {
  const thing = new Thing()
  expect(import_('{"type": "Thing"}')).toEqual(thing)
  expect(import_({type: 'Thing'})).toEqual(thing)
  expect(import_(thing)).toEqual(thing)

  expect(import_('{"type": "Thing"}', 'application/ld+json')).toEqual(thing)
  expect(() => import_("", 'foo/bar')).toThrow(/^Unhandled import format: foo\/bar/)
})

test('import:Person', () => {
  expect(import_('{"type": "Person", "name": "Jane"}')).toEqual(new Person({name: "Jane"}))
})
