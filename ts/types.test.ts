import { person, paragraph, link } from './types'

describe('Schema factory functions', () => {
  test('no required properties', () => {
    expect(person()).toMatchObject({ type: 'Person' })
  })

  test('one required property', () => {
    expect(
      paragraph({ content: ['The content of the paragraph'] })
    ).toMatchObject({
      type: 'Paragraph',
      content: ['The content of the paragraph']
    })
  })

  test('more than one required property', () => {
    expect(
      link({
        content: ['The content of the link'],
        target: 'https://example.org'
      })
    ).toMatchObject({
      type: 'Link',
      content: ['The content of the link'],
      target: 'https://example.org'
    })
  })

  test('it filters undefined values from properties', () => {
    const actual = person({
      honorificPrefix: 'Sir',
      honorificSuffix: undefined,
      givenNames: ['Isaac'],
      familyNames: ['Newton'],
      memberOf: undefined
    })

    expect(actual).toMatchObject({
      type: 'Person',
      familyNames: ['Newton'],
      givenNames: ['Isaac'],
      honorificPrefix: 'Sir'
    })
  })
})
