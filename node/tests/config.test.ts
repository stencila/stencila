import { config } from '../lib'

describe('config', () => {
  const conf = config.read()

  expect(conf).toEqual(
    expect.objectContaining({
      logging: expect.objectContaining({}),
      plugins: expect.objectContaining({}),
      serve: expect.objectContaining({}),
      upgrade: expect.objectContaining({}),
    })
  )

  test('validate', () => {
    expect(config.validate(conf)).toBe(true)
    try {
      // @ts-ignore
      config.validate({ logging: { file: { level: 'foo' } } })
    } catch (error) {
      expect(error.toString()).toMatch(
        'unknown variant `foo`, expected one of `debug`, `info`, `warn`, `error`, `never` at line 1 column 33'
      )
    }
  })

  test('set', () => {
    expect(config.set(conf, 'upgrade.auto', '1 week')).toEqual({
      ...conf,
      upgrade: {
        ...conf.upgrade,
        auto: '1 week',
      },
    })
    try {
      config.set(conf, 'upgrade.auto', 'foo bar')
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
    config.reset(conf, 'all')
    config.reset(conf, 'logging')
    try {
      // @ts-ignore
      config.reset(conf, 'foo')
    } catch (error) {
      // @ts-ignore
      expect(error.toString()).toMatch(
        'No top level configuration property named: foo'
      )
    }
  })
})
