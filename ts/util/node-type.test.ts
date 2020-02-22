import { nodeType } from './node-type'

test('nodeType', () => {
  expect(nodeType(null)).toBe('Null')
  expect(nodeType(true)).toBe('Boolean')
  expect(nodeType(false)).toBe('Boolean')
  expect(nodeType(42)).toBe('Number')
  expect(nodeType(3.14)).toBe('Number')
  expect(nodeType('str')).toBe('Text')
  expect(nodeType([])).toBe('Array')
  expect(nodeType({})).toBe('Object')
  expect(nodeType({ type: 'Person' })).toBe('Person')
})
