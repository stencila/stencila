import {
  isA,
  isInlineContent,
  isInlineEntity,
  isPrimitive,
  isType,
  nodeIs,
  typeIs
} from './guards'
import { TypeMap } from './type-map'
import { blockContentTypes, inlineContentTypes } from './type-maps'

const primitives = [null, true, false, NaN, 2, 'string']

const typeMap = ({
  someType: 'someType',
  myCustomType: 'myCustomType'
} as unknown) as TypeMap

describe('typeIs', () => {
  it('finds the given type', () => {
    // @ts-ignore
    expect(typeIs(typeMap)(typeMap.myCustomType)).toBe(true)
  })

  it('returns false when queried type is not in the type map', () => {
    expect(typeIs(typeMap)('otherType')).toBe(false)
  })
})

describe('nodeIs', () => {
  test('it returns false for undefined values', () =>
    // @ts-ignore
    expect(nodeIs(typeMap)(undefined)).toBe(false))

  test('returns false for primitive values', () => {
    primitives.map(node => expect(nodeIs(typeMap)(node)).toBe(false))
    expect.assertions(primitives.length)
  })

  test('it returns false for empty Arrays', () =>
    expect(nodeIs(typeMap)([])).toBe(false))

  test('it returns false for Arrays with content', () =>
    expect(nodeIs(typeMap)([{ type: 'someType' }])).toBe(false))

  test('it returns false for Objects without a "type" key', () =>
    expect(nodeIs(typeMap)({ content: ['someContent'] })).toBe(false))

  test('it returns false for Objects containing a "type" key not found in the typeMap', () =>
    expect(nodeIs(typeMap)({ type: 'someOtherType' })).toBe(false))

  test('it returns true for Objects containing a "type" key found in the typeMap', () =>
    // @ts-ignore
    expect(nodeIs(typeMap)({ type: typeMap.someType })).toBe(true))
})

describe('isA', () => {
  const person = { type: 'Person' }
  const para = { type: 'Paragraph', content: [] }

  test('it returns false for undefined types', () => {
    // This is a compile error too
    // @ts-ignore
    expect(isA(person, 'Foo')).toBe(false)
  })

  test('it returns true for the right type', () => {
    expect(isA('Person', person)).toBe(true)
    expect(isA('Paragraph', para)).toBe(true)
  })

  test('it returns false for the wrong type', () => {
    expect(isA('Person', para)).toBe(false)
    expect(isA('Person', null)).toBe(false)
    expect(isA('Person', true)).toBe(false)
    expect(isA('Person', 1.0)).toBe(false)
    expect(isA('Person', [])).toBe(false)
    expect(isA('Person', { type: 'Foo' })).toBe(false)
  })
})

describe('isType', () => {
  const person = { type: 'Person' }
  const para = { type: 'Paragraph', content: [] }

  test('it returns false for undefined types', () => {
    // This is a compile error too
    // @ts-ignore
    expect(isType('Foo')(person)).toBe(false)
  })

  test('it returns true for the right type', () => {
    expect(isType('Person')(person)).toBe(true)
  })

  test('it returns false for the wrong type', () => {
    expect(isType('Person')(para)).toBe(false)
  })
})

describe('isPrimitive', () => {
  test('returns true for primitive values', () => {
    primitives.map(node => expect(isPrimitive(node)).toBe(true))
    expect.assertions(primitives.length)
  })

  test('it returns false for empty Arrays', () =>
    expect(isPrimitive([])).toBe(false))

  test('it returns false for Arrays with content', () =>
    expect(isPrimitive([{ type: 'someType' }])).toBe(false))

  test('it returns false for Objects', () =>
    expect(isPrimitive({ type: 'someOtherType' })).toBe(false))
})

describe('isInlineEntity', () => {
  test('returns false for primitive values', () => {
    primitives.map(node => expect(isInlineEntity(node)).toBe(false))
    expect.assertions(primitives.length)
  })

  test('it returns false for empty Arrays', () =>
    expect(isInlineEntity([])).toBe(false))

  test('it returns false for Arrays with content', () =>
    expect(isInlineEntity([{ type: 'someType' }])).toBe(false))

  test('it returns false for Objects containing a "type" key not found in the typeMap', () =>
    expect(isInlineEntity({ type: 'someOtherType' })).toBe(false))

  test('it returns false for BlockContent type', () =>
    expect(isInlineEntity({ type: 'Paragraph' })).toBe(false))

  test('it returns true for Objects containing a "type" key found in the typeMap', () =>
    expect(isInlineEntity({ type: 'CodeExpression' })).toBe(true))
})

describe('isInlineContent', () => {
  test('returns true for primitive types', () => {
    primitives.map(type => expect(isInlineContent(type)).toBe(true))
  })

  test('returns false for BlockContent types ', () => {
    Object.values(blockContentTypes).map(type => {
      expect(isInlineContent({ type })).toBe(false)
    })

    expect.assertions(Object.values(blockContentTypes).length)
  })

  test('returns true for InlineContent types ', () => {
    Object.values(inlineContentTypes).map(type => {
      expect(isInlineContent({ type })).toBe(true)
    })

    expect.assertions(Object.values(inlineContentTypes).length)
  })
})
