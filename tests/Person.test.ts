import { mutate } from '../util'

describe('mutate', () => {
  it('coerces properties', () => {
    expect(
      mutate(
        {
          givenNames: 'John',
          familyNames: 'Smith',
          // Unfortunately it is not possible to
          // coerce an an object to an array of objects
          // so we must always have an array here
          affiliations: [
            {
              name: 'University of Beep, Boop'
            }
          ]
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      givenNames: ['John'],
      familyNames: ['Smith'],
      affiliations: [
        {
          type: 'Organization',
          name: 'University of Beep, Boop'
        }
      ]
    })
  })

  it('renames and coerces property aliases', () => {
    expect(
      mutate(
        {
          firstName: 'John',
          lastName: 'Smith'
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      givenNames: ['John'],
      familyNames: ['Smith']
    })
    expect(
      mutate(
        {
          givenName: 'John',
          familyName: 'Smith'
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      givenNames: ['John'],
      familyNames: ['Smith']
    })
  })
})
