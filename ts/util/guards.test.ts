/* eslint-disable @typescript-eslint/ban-ts-comment */

import {
  audioObject,
  blockContentTypes,
  entity,
  imageObject,
  inlineContentTypes,
  mediaObject,
  person,
} from '../types'
import {
  isA,
  isBlockContent,
  isEntity,
  isInlineContent,
  isMember,
  isPrimitive,
  isType,
} from './guards'

const primitives = [
  null,
  true,
  false,
  NaN,
  2,
  'string',
  [],
  [1, 2, 3],
  {},
  { 'not-type': 'foo' },
]

describe('isPrimitive', () => {
  test.each(primitives)('returns true for primitive value of "%s"', (node) => {
    expect(isPrimitive(node)).toBe(true)
  })

  test('it returns false for Object with type property', () =>
    expect(isPrimitive({ type: 'Emphasis' })).toBe(false))
})

describe('isEntity', () => {
  test('it returns true for any object with type property', () =>
    expect(isEntity({ type: 'foo' })).toBe(true))

  test.each(primitives)('returns false for primitive value of "%s"', (node) => {
    expect(isEntity(node)).toBe(false)
  })
})

describe('isMember', () => {
  const isMedia = isMember('MediaObjectTypes')

  it('returns true for nodes of the type and its descendants', () => {
    expect(isMedia(mediaObject({ contentUrl: '' }))).toBe(true)
    expect(isMedia(audioObject({ contentUrl: '' }))).toBe(true)
    expect(isMedia(imageObject({ contentUrl: '' }))).toBe(true)
  })

  it('returns false for nodes that are of an ancestor type', () => {
    expect(isMedia(entity())).toBe(false)
  })

  it('returns false for unrelated node types', () => {
    expect(isMedia(person())).toBe(false)
  })

  test.each(primitives)('returns false for primitive value of "%s"', (node) => {
    expect(isEntity(node)).toBe(false)
  })
})

describe('isA', () => {
  const person = { type: 'Person' }
  const para = { type: 'Paragraph', content: [] }

  test('it returns false for undefined types', () => {
    // @ts-expect-error
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
    // @ts-expect-error
    expect(isType('Foo')(person)).toBe(false)
  })

  test('it returns true for the right type', () => {
    expect(isType('Person')(person)).toBe(true)
  })

  test('it returns false for the wrong type', () => {
    expect(isType('Person')(para)).toBe(false)
  })
})

describe('isInlineContent', () => {
  test.each(primitives)('returns true for primitive value of "%s"', (type) => {
    expect(isInlineContent(type)).toBe(true)
  })

  test.each(Object.values(blockContentTypes))(
    'returns false for BlockContent type of "%s"',
    (type) => {
      expect(isInlineContent({ type })).toBe(false)
    }
  )

  test.each(Object.values(inlineContentTypes))(
    'returns true for InlineContent type of "%s"',
    (type) => {
      expect(isInlineContent({ type })).toBe(true)
    }
  )
})

describe('isBlockContent', () => {
  test.each(primitives)('returns false for primitive value of "%s"', (type) => {
    expect(isBlockContent(type)).toBe(false)
  })

  test.each(Object.values(blockContentTypes))(
    'returns true for BlockContent type of "%s"',
    (type) => {
      expect(isBlockContent({ type })).toBe(true)
    }
  )

  test.each(Object.values(inlineContentTypes))(
    'returns false for InlineContent type of "%s"',
    (type) => {
      expect(isBlockContent({ type })).toBe(false)
    }
  )
})
