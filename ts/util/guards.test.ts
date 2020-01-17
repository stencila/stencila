import { Code, CodeBlock, CreativeWork } from '../types'
import {
  isA,
  isInlineContent,
  isInlineEntity,
  isInstanceOf,
  isPrimitive,
  isType,
  nodeIs,
  typeIs
} from './guards'
import { TypeMap } from './type-map'
import {
  blockContentTypes,
  codeBlockTypes,
  codeTypes,
  creativeWorkTypes,
  inlineContentTypes
} from './type-maps'

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

  test.each(primitives)('returns false for primitive value of "%s"', node => {
    expect(nodeIs(typeMap)(node)).toBe(false)
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
  test.each(primitives)('returns true for primitive value of "%s"', node => {
    expect(isPrimitive(node)).toBe(true)
  })

  test('it returns false for empty Arrays', () =>
    expect(isPrimitive([])).toBe(false))

  test('it returns false for Arrays with content', () =>
    expect(isPrimitive([{ type: 'someType' }])).toBe(false))

  test('it returns false for Objects', () =>
    expect(isPrimitive({ type: 'someOtherType' })).toBe(false))
})

describe('isInlineEntity', () => {
  test.each(primitives)('returns false for primitive value of "%s"', node => {
    expect(isInlineEntity(node)).toBe(false)
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
  test.each(primitives)('returns true for primitive value of "%s"', type => {
    expect(isInlineContent(type)).toBe(true)
  })

  test.each(Object.values(blockContentTypes))(
    'returns false for BlockContent type of "%s"',
    type => {
      expect(isInlineContent({ type })).toBe(false)
    }
  )

  test.each(Object.values(inlineContentTypes))(
    'returns true for InlineContent type of "%s"',
    type => {
      expect(isInlineContent({ type })).toBe(true)
    }
  )
})

describe('handle descendant type matching', () => {
  test('it matches the descendent schema', () => {
    expect(
      isInstanceOf<Code>(codeTypes, { type: 'Code' })
    ).toBe(true)

    expect(
      isInstanceOf<Code>(codeTypes, { type: 'CodeFragment' })
    ).toBe(true)
  })

  test('it does not match against parent schema', () => {
    expect(
      isInstanceOf<CodeBlock>(codeBlockTypes, { type: 'Code' })
    ).toBe(false)

    expect(
      isInstanceOf<CodeBlock>(codeBlockTypes, { type: 'Code' })
    ).toBe(false)
  })

  test('it does not match across schemas', () => {
    expect(
      isInstanceOf<CreativeWork>(creativeWorkTypes, {
        type: 'CodeFragment'
      })
    ).toBe(false)
  })
})
