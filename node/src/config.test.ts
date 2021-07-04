import {
  get,
  validate,
  set,
  resetProperty,
  schema,
  setProperty,
} from './config'

test('schema', () => {
  expect(schema()).toEqual(
    expect.objectContaining({
      $schema: 'https://json-schema.org/draft/2019-09/schema',
      $id: 'Config',
      type: 'object',
      properties: expect.objectContaining({
        projects: expect.objectContaining({
          title: 'Projects',
          description: 'Configuration settings for project defaults',
        }),
        logging: expect.objectContaining({}),
        plugins: expect.objectContaining({}),
        serve: expect.objectContaining({}),
        upgrade: expect.objectContaining({}),
      }),
    })
  )
})

test('get', () => {
  const conf = get()
  expect(conf).toEqual(
    expect.objectContaining({
      logging: expect.objectContaining({}),
      plugins: expect.objectContaining({}),
      serve: expect.objectContaining({}),
      upgrade: expect.objectContaining({}),
    })
  )
})

test('set', () => {
  const conf = set({})
  expect(conf).toEqual(get())
})

test('validate', () => {
  const conf = get()
  expect(validate(conf)).toBe(true)

  try {
    // @ts-ignore
    validate({ logging: { file: { level: 'foo' } } })
  } catch (error) {
    expect(error.toString()).toMatch(
      'unknown variant `foo`, expected one of `trace`, `debug`, `info`, `warn`, `error`, `never` at line 1 column 33'
    )
  }
})

test('setProperty', () => {
  const conf = get()
  expect(setProperty('upgrade.auto', '1 week')).toEqual({
    ...conf,
    upgrade: {
      ...conf.upgrade,
      auto: '1 week',
    },
  })
  try {
    setProperty('upgrade.auto', 'foo bar')
  } catch (error) {
    expect(error.toString()).toMatch(
      `Error: upgrade.auto: Not a valid duration`
    )
  }
})

test('resetProperty', () => {
  resetProperty('all')
  resetProperty('logging')
  try {
    // @ts-ignore
    resetProperty('foo')
  } catch (error) {
    // @ts-ignore
    expect(error.toString()).toMatch(
      'No top level configuration property named: foo'
    )
  }
})
