import { nodeType } from './node-type'

test('nodeType', () => {
  expect(nodeType(null)).toBe('null')
  expect(nodeType(true)).toBe('boolean')
  expect(nodeType(false)).toBe('boolean')
  expect(nodeType(42)).toBe('number')
  expect(nodeType(3.14)).toBe('number')
  expect(nodeType('str')).toBe('string')
  expect(nodeType([])).toBe('array')
  expect(nodeType({})).toBe('object')
  expect(nodeType({ type: 'Person' })).toBe('Person')
})
