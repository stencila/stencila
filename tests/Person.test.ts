import { create, validate, mutate } from '../util'
import { parse } from '../schema/organization/Person'

describe('validate', () => {
  it('throws for invalid emails', () => {
    expect(() =>
      validate(
        {
          type: 'Person',
          emails: ['pete@example.com', 'pete_at_example_com']
        },
        'Person'
      )
    ).toThrow('/emails/1: format should match format "email"')
  })
})

describe('parse', () => {
  it('works', () => {
    let person = create('Person')

    person.familyNames = ['Jones']
    expect(parse('Jones')).toEqual(person)

    person.givenNames = ['Jane', 'Jill']
    expect(parse('Jane Jill Jones')).toEqual(person)

    person.honorificPrefix = 'Dr'
    expect(parse('Dr Jane Jill Jones')).toEqual(person)

    person.honorificSuffix = 'PhD'
    expect(parse('Dr Jane Jill Jones PhD')).toEqual(person)

    person.emails = ['jane@example.com']
    expect(parse('Dr Jane Jill Jones PhD <jane@example.com>')).toEqual(person)

    person.url = 'http://example.com/jane'
    expect(
      parse(
        'Dr Jane Jill Jones PhD <jane@example.com> (http://example.com/jane)'
      )
    ).toEqual(person)
  })

  it('throws', () => {
    expect(() => parse('')).toThrow(/^Unable to parse string \"\" as a person$/)
    expect(() => parse('#@&%')).toThrow(
      /^Unable to parse string \"#@&%\" as a person$/s
    )
  })
})

describe('mutate', () => {
  it('coerces properties', () => {
    expect(
      mutate(
        {
          givenNames: 'John Tom',
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
      givenNames: ['John', 'Tom'],
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
          firstNames: 'John Tom',
          lastName: 'Smith'
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      givenNames: ['John', 'Tom'],
      familyNames: ['Smith']
    })

    expect(
      mutate(
        {
          givenName: 'Jane',
          surnames: 'Doe Smith'
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      givenNames: ['Jane'],
      familyNames: ['Doe', 'Smith']
    })
  })

  it('parses strings into people', () => {
    expect(
      mutate(
        {
          authors: [
            'John Smith',
            'Dr Jane Jones PhD <jane@example.com>',
            'Jones, Jack (http://example.com/jack)'
          ]
        },
        'CreativeWork'
      )
    ).toEqual({
      type: 'CreativeWork',
      authors: [
        {
          type: 'Person',
          givenNames: ['John'],
          familyNames: ['Smith']
        },
        {
          type: 'Person',
          honorificPrefix: 'Dr',
          givenNames: ['Jane'],
          familyNames: ['Jones'],
          honorificSuffix: 'PhD',
          emails: ['jane@example.com']
        },
        {
          type: 'Person',
          givenNames: ['Jack'],
          familyNames: ['Jones'],
          url: 'http://example.com/jack'
        }
      ]
    })
  })

  it('throws if string can not be parsed', () => {
    expect(() =>
      mutate({ authors: ['John Smith', '#@&%', 'Jones, Jane'] }, 'CreativeWork')
    ).toThrow(
      '/authors/1: parser error when parsing using "person": Unable to parse string "#@&%" as a person'
    )
  })
})
