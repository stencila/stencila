import Person from '../src/Person'

describe('Person', () => {

  test('type', () => {
    const person = new Person()
    expect(person.type).toEqual('Person')
  })

  test('fromText', () => {
    let person
    
    person = Person.fromText('Joe')
    //expect(person.name).toEqual('Joe')
    expect(person.givenNames).toEqual(['Joe'])
    expect(person.familyNames).toEqual([])
    
    person = Person.fromText('Joe Bloggs')
    expect(person.name).toEqual('Joe Bloggs')
    expect(person.givenNames).toEqual(['Joe'])
    expect(person.familyNames).toEqual(['Bloggs'])

    person = Person.fromText('Joe Bloggs <joe@example.com>')
    expect(person.name).toEqual('Joe Bloggs')
    expect(person.givenNames).toEqual(['Joe'])
    expect(person.familyNames).toEqual(['Bloggs'])
    expect(person.emails).toEqual(['joe@example.com'])

    person = Person.fromText('Joe Bloggs <joe@example.com> (https://example.com/joe)')
    expect(person.name).toEqual('Joe Bloggs')
    expect(person.givenNames).toEqual(['Joe'])
    expect(person.familyNames).toEqual(['Bloggs'])
    expect(person.emails).toEqual(['joe@example.com'])
    expect(person.urls).toEqual(['https://example.com/joe'])

    person = Person.fromText(' Joe    Bloggs   ')
    expect(person.name).toEqual('Joe Bloggs')
  })

})
