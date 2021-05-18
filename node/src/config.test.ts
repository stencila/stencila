import { read, validate, set, reset, schema } from './config'

describe('config', () => {
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
        'unknown variant `foo`, expected one of `trace`, `debug`, `info`, `warn`, `error`, `never` at line 1 column 33'
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
