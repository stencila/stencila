import { article, organization, person } from './types'

describe('Schema factory functions', () => {
  test('it creates a valid Person node', () => {
    expect(person()).toMatchObject({ type: 'Person' })
  })

  test('it creates a valid Article node', () => {
    expect(
      article([organization({ name: 'Stencila' })], 'Testing')
    ).toMatchObject({
      type: 'Article',
      title: 'Testing',
      authors: [
        {
          name: 'Stencila',
          type: 'Organization'
        }
      ]
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
