import { list, install, uninstall, upgrade } from './plugins'

describe('plugins', () => {
  test('list', () => {
    expect(list()).toEqual(expect.arrayContaining([]))
  })

  test.skip('install', () => {
    expect(install('javascript')).toEqual(expect.arrayContaining([]))
  })

  test('uninstall', () => {
    expect(uninstall('javascript')).toEqual(expect.arrayContaining([]))
  })

  test.skip('upgrade', () => {
    expect(upgrade('javascript')).toEqual(expect.arrayContaining([]))
  })
})
