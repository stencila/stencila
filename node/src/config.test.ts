import { read, validate, set, reset, schema } from './config'

describe('config', () => {
  test('schema', () => {
    expect(schema()).toEqual(
      expect.objectContaining({
        $schema: 'http://json-schema.org/draft-07/schema#',
        title: 'Config',
        type: 'object',
        definitions: expect.objectContaining({}),
        properties: expect.objectContaining({
          logging: expect.objectContaining({}),
          plugins: expect.objectContaining({}),
          serve: expect.objectContaining({}),
          upgrade: expect.objectContaining({}),
        }),
      })
    )
  })

  const conf = read()

  expect(conf).toEqual(
    expect.objectContaining({
      logging: expect.objectContaining({}),
      plugins: expect.objectContaining({}),
      serve: expect.objectContaining({}),
      upgrade: expect.objectContaining({}),
    })
  )

  test('validate', () => {
    expect(validate(conf)).toBe(true)
    try {
      // @ts-ignore
      validate({ logging: { file: { level: 'foo' } } })
    } catch (error) {
      expect(error.toString()).toMatch(
        'unknown variant `foo`, expected one of `debug`, `info`, `warn`, `error`, `never` at line 1 column 33'
      )
    }
  })

  test('set', () => {
    expect(set(conf, 'upgrade.auto', '1 week')).toEqual({
      ...conf,
      upgrade: {
        ...conf.upgrade,
        auto: '1 week',
      },
    })
    try {
      set(conf, 'upgrade.auto', 'foo bar')
    } catch (error) {
      expect(error.toString()).toMatch(`Invalid configuration value/s:

{
  "upgrade": {
    "auto": [
      {
        "code": "invalid_duration_string",
        "message": "Not a valid duration",
        "params": {
          "value": "foo bar"
        }
      }
    ]
  }
}`)
    }
  })

  test('reset', () => {
    reset(conf, 'all')
    reset(conf, 'logging')
    try {
      // @ts-ignore
      reset(conf, 'foo')
    } catch (error) {
      // @ts-ignore
      expect(error.toString()).toMatch(
        'No top level configuration property named: foo'
      )
    }
  })
})
