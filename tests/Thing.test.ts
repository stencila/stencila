import Thing from '../src/types/Thing'

test('constructor', () => {
  const thing1 = new Thing()
  expect(thing1.name).toEqual('')

  const thing2 = new Thing({
    name: 'thing2',
    description: 'The second thing'
  })
  expect(thing2.name).toEqual('thing2')
  expect(thing2.description).toEqual('The second thing')

  const thing3 = new Thing({
    name: 'thing3',
    foo: 'bar'
  })
  expect(thing3.name).toEqual('thing3')
  // @ts-ignore
  expect(thing3.foo).toEqual(undefined)
})

test('type', () => {
  const thing = new Thing()
  expect(thing.type).toEqual('Thing')
})
