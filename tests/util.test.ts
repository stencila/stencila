import { assert, create, is, cast, type, valid, validate } from '../util'

describe('create', () => {
  it('works with different types', () => {
    expect(create('Thing')).toEqual({
      type: 'Thing'
    })
    expect(create('CreativeWork')).toEqual({
      type: 'CreativeWork'
    })
  })

  it('works with initial values', () => {
    expect(
      create('Thing', {
        name: 'thing1',
        url: 'http://example.com/thing1'
      })
    ).toEqual({
      type: 'Thing',
      name: 'thing1',
      url: 'http://example.com/thing1'
    })
  })

  it('throws with unknown types', () => {
    // In Typescript this error is caught at compile time, so ts-ignore it
    // @ts-ignore
    expect(() => create('Foo')).toThrow(/^No schema for type "Foo".$/)
  })

  it('throws when wrong initial values', () => {
    expect(() => create('Thing', { foo: 'Foo' })).toThrow(
      'Property foo is not expected to be here'
    )
    expect(() => create('Thing', { type: 'Foo' })).toThrow(
      'type should be equal to one of the allowed values: Thing'
    )
  })

  it('does not throw when wrong initial values and no validation', () => {
    expect(create('Thing', { foo: 'invalid' }, false)).toEqual({
      type: 'Thing',
      foo: 'invalid'
    })
  })
})

test('type', () => {
  expect(type(undefined)).toBe('undefined')
  expect(type(null)).toBe('null')
  expect(type(true)).toBe('boolean')
  expect(type(0)).toBe('number')
  expect(type('0')).toBe('string')
  expect(type([])).toBe('array')
  expect(type({})).toBe('object')
  expect(type({ type: 'Thing' })).toBe('Thing')
})

test('is', () => {
  expect(is(undefined, 'undefined')).toEqual(true)
  expect(is(null, 'null')).toEqual(true)
  expect(is(true, 'boolean')).toEqual(true)
  expect(is(0, 'number')).toEqual(true)
  expect(is('0', 'string')).toEqual(true)
  expect(is([], 'array')).toEqual(true)
  expect(is({}, 'object')).toEqual(true)
  expect(is({ type: 'Thing' }, 'Thing')).toEqual(true)
  expect(is({ type: 'Foo' }, 'Thing')).toEqual(false)
})

describe('assert', () => {
  it('works', () => {
    expect(assert(null, 'null')).toEqual(true)
    expect(assert(42, ['number', 'Thing'])).toEqual(true)
    expect(assert({ type: 'Thing' }, 'Thing')).toEqual(true)
    expect(assert({ type: 'Thing' }, ['Thing', 'Foo'])).toEqual(true)
  })

  it('throws on wrong type', () => {
    expect(() => assert(null, 'Thing')).toThrow(
      /type is "null" but expected "Thing"/
    )
    expect(() => assert(42, ['Thing', 'CreativeWork'])).toThrow(
      /type is "number" but expected "Thing\|CreativeWork"/
    )
    expect(() => assert({ type: 'Datatable' }, 'Thing')).toThrow(
      /type is "Datatable" but expected "Thing"/
    )
    expect(() => assert({ type: 'Foo' }, ['Thing', 'CreativeWork'])).toThrow(
      /type is "Foo" but expected "Thing\|CreativeWork"/
    )
  })
})

describe('cast', () => {
  it('works', () => {
    expect(cast({}, 'Thing')).toEqual({
      type: 'Thing'
    })
    expect(cast({ type: 'Thing' }, 'Thing')).toEqual({
      type: 'Thing'
    })
    expect(cast({ type: 'Thing', authors: [] }, 'CreativeWork')).toEqual({
      type: 'CreativeWork',
      authors: []
    })
    expect(
      cast(
        {
          type: 'Thing',
          authors: [{ type: 'Person', givenNames: ['Jack'] }]
        },
        'Article'
      )
    ).toEqual({
      type: 'Article',
      authors: [{ type: 'Person', givenNames: ['Jack'] }]
    })
  })

  it('throws on wrong property type', () => {
    expect(() => cast({ name: 42 }, 'Thing')).toThrow(
      'name: type should be string'
    )
    expect(() => cast({ url: [] }, 'Thing')).toThrow(
      'url: type should be string'
    )
  })

  it('throws on additional property', () => {
    expect(() => cast({ foo: 'Bar' }, 'Thing')).toThrow(
      'Property foo is not expected to be here'
    )
  })

  it('throws on missing property', () => {
    expect(() => cast({ type: 'Thing' }, 'Article')).toThrow(
      "should have required property 'authors'"
    )
  })
})

describe('validate', () => {
  it('throws for non-objects', () => {
    expect(() => validate(null, 'Thing')).toThrow(/^: type should be object$/)
    expect(() => validate(42, 'Thing')).toThrow(/^: type should be object$/)
  })

  it('throws for missing properties', () => {
    expect(() => validate({}, 'Thing')).toThrow(/^ should have required property 'type'$/)
    expect(() => validate({type: 'Article'}, 'Article')).toThrow(/^ should have required property 'authors'$/)
  })

  it('throws on type with no schema', () => {
    // In Typescript this error is caught at compile time, so ts-ignore it
    // @ts-ignore
    expect(() => cast({}, 'Foo')).toThrow(/^No schema for type "Foo".$/)
  })
})

describe('valid', () => {
  it('works', () => {
    expect(valid(null, 'Thing')).toBe(false)
    expect(valid({type: 'Thing'}, 'Thing')).toBe(true)
  })
})
