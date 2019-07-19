import {
  blockContentTypes,
  inlineContentTypes,
  isInlineContent,
  isInlineNonPrimitive,
  isInlinePrimitive,
  isOfType,
  nodeIsOfType,
  TypeMap
} from '../guards'

const primitives = [null, true, false, NaN, 2, 'string']

const typeMap = ({
  someType: 'someType',
  myCustomType: 'myCustomType'
} as unknown) as TypeMap

describe('isOfType', () => {
  it('finds the given type', () => {
    // @ts-ignore
    expect(isOfType(typeMap)(typeMap.myCustomType)).toBe(true)
  })

  it('returns false when queried type is not in the type map', () => {
    expect(isOfType(typeMap)('otherType')).toBe(false)
  })
})

describe('isNode', () => {
  test('it returns false for undefined values', () =>
    // @ts-ignore
    expect(nodeIsOfType(typeMap)(undefined)).toBe(false))

  test('returns false for primitive values', () => {
    primitives.map(node => expect(nodeIsOfType(typeMap)(node)).toBe(false))
    expect.assertions(primitives.length)
  })

  test('it returns false for empty Arrays', () =>
    expect(nodeIsOfType(typeMap)([])).toBe(false))

  test('it returns false for Arrays with content', () =>
    expect(nodeIsOfType(typeMap)([{ type: 'someType' }])).toBe(false))

  test('it returns false for Objects without a "type" key', () =>
    expect(nodeIsOfType(typeMap)({ content: ['someContent'] })).toBe(false))

  test('it returns false for Objects containing a "type" key not found in the typeMap', () =>
    expect(nodeIsOfType(typeMap)({ type: 'someOtherType' })).toBe(false))

  test('it returns true for Objects containing a "type" key found in the typeMap', () =>
    // @ts-ignore
    expect(nodeIsOfType(typeMap)({ type: typeMap.someType })).toBe(true))
})

describe('isInlinePrimitive', () => {
  test('returns true for primitive values', () => {
    primitives.map(node => expect(isInlinePrimitive(node)).toBe(true))
    expect.assertions(primitives.length)
  })

  test('it returns false for empty Arrays', () =>
    expect(isInlinePrimitive([])).toBe(false))

  test('it returns false for Arrays with content', () =>
    expect(isInlinePrimitive([{ type: 'someType' }])).toBe(false))

  test('it returns false for Objects', () =>
    expect(isInlinePrimitive({ type: 'someOtherType' })).toBe(false))
})

describe('isInlineNonPrimitive', () => {
  test('returns false for primitive values', () => {
    primitives.map(node => expect(isInlineNonPrimitive(node)).toBe(false))
    expect.assertions(primitives.length)
  })

  test('it returns false for empty Arrays', () =>
    expect(isInlineNonPrimitive([])).toBe(false))

  test('it returns false for Arrays with content', () =>
    expect(isInlineNonPrimitive([{ type: 'someType' }])).toBe(false))

  test('it returns false for Objects containing a "type" key not found in the typeMap', () =>
    expect(isInlineNonPrimitive({ type: 'someOtherType' })).toBe(false))

  test('it returns false for BlockContent type', () =>
    expect(isInlineNonPrimitive({ type: 'Paragraph' })).toBe(false))

  test('it returns true for Objects containing a "type" key found in the typeMap', () =>
    expect(isInlineNonPrimitive({ type: 'CodeExpr' })).toBe(true))
})

describe('isInlineContent', () => {
  test('returns true for primitive types', () => {
    primitives.map(type => expect(isInlineContent(type)).toBe(true))
  })

  // TODO: Revisit/revise Code schema (Code, CodeBlock, CodeExpression, & CodeChunk)
  test.skip('returns false for BlockContent types ', () => {
    Object.values(blockContentTypes).map(type => {
      expect(isInlineContent(type)).toBe(false)
    })

    expect.assertions(Object.values(blockContentTypes).length)
  })

  test('returns true for InlineContent types ', () => {
    Object.values(inlineContentTypes).map(type => {
      expect(isInlineContent(type)).toBe(true)
    })

    expect.assertions(Object.values(inlineContentTypes).length)
  })
})
