import {
  assert,
  create,
  is,
  cast,
  type,
  valid,
  validate,
  mutate
} from '../util'

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
    expect(create('Thing', { foo: 'invalid' }, 'none')).toEqual({
      type: 'Thing',
      foo: 'invalid'
    })
  })

  it('will mutate initial value to conform to schema', () => {
    expect(create('Thing', { name: 42, foo: 'invalid' }, 'mutate')).toEqual({
      type: 'Thing',
      name: '42'
    })
  })
})

test('type', () => {
  expect(type(undefined)).toBe('undefined')
  expect(type(null)).toBe('null')
  expect(type(true)).toBe('boolean')
  expect(type(0)).toBe('number')
  expect(type(NaN)).toBe('number')
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
    expect(() => validate({}, 'Thing')).toThrow(
      /^ should have required property 'type'$/
    )
    expect(() => validate({ type: 'Article' }, 'Article')).toThrow(
      /^ should have required property 'authors'$/
    )
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
    expect(valid({ type: 'Thing' }, 'Thing')).toBe(true)
  })
})

describe('mutate', () => {
  it('will add type property', () => {
    expect(mutate({}, 'Person')).toEqual({
      type: 'Person'
    })
    expect(
      mutate(
        {
          type: 'Foo',
          name: 'John'
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      name: 'John'
    })
  })

  it('will coerce types', () => {
    expect(
      mutate(
        {
          name: 42
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      name: '42'
    })
    expect(
      mutate(
        {
          name: null
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      name: ''
    })
  })

  it('will coerce arrays to scalars', () => {
    expect(
      mutate(
        {
          type: 'Person',
          name: [42]
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      name: '42'
    })
  })

  it('will coerce scalars to arrays', () => {
    expect(
      mutate(
        {
          type: 'Person',
          givenNames: 'Jane'
        },
        'Person'
      )
    ).toEqual({
      type: 'Person',
      givenNames: ['Jane']
    })
  })

  it('will add default values for missing properties', () => {
    expect(mutate({}, 'Thing')).toEqual({
      type: 'Thing'
    })
  })

  it('will remove additional properties', () => {
    expect(
      mutate(
        {
          favoriteColor: 'red'
        },
        'Person'
      )
    ).toEqual({
      type: 'Person'
    })
  })

  it('throws an error if unable to coerce data, or data is otherwise invalid', () => {
    expect(() =>
      mutate(
        {
          name: {}
        },
        'Person'
      )
    ).toThrow('name: type should be string')

    expect(() =>
      mutate(
        {
          url: 'foo'
        },
        'Person'
      )
    ).toThrow('url: format should match format "uri"')
  })

  it('throws an error if invalid type', () => {
    // @ts-ignore
    expect(() => mutate({}, 'Foo')).toThrow(/^No schema for type "Foo".$/)
  })

  it('has no side effects', () => {
    const inp: any = {
      name: 42,
      authors: [{ type: 'Person', givenNames: 'Jane' }]
    }
    const out = mutate(inp, 'Article')

    // The original object should be unchanged
    expect(inp.type).toBeUndefined()
    expect(inp.name).toEqual(42)
    const inpPerson = inp.authors[0]
    expect(inpPerson.givenNames).toEqual('Jane')

    // The new object has the changes made
    expect(out.type).toEqual('Article')
    expect(out.name).toEqual('42')
    const outPerson = cast(out.authors[0], 'Person')
    expect(outPerson.givenNames).toEqual(['Jane'])
  })
})
