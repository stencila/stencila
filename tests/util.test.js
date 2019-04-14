/**
 * A few ancillary tests in Javascript to check and illustrate
 * usage there (i.e. with no compile time checking)
 */

const { create, type, is } = require('../util')

test('create', () => {
  expect(create('Thing')).toEqual({ type: 'Thing' })
  // This will create a compile time error in Typescript
  expect(() => create('Foo')).toThrow(/^No schema for type "Foo".$/)
})

test('type', () => {
  expect(type(null)).toBe('null')
  expect(type({})).toBe('object')
  expect(type(create('Thing'))).toBe('Thing')
})

test('is', () => {
  expect(is(create('Thing'), 'Thing')).toBe(true)
})
