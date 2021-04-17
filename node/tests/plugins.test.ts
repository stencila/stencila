import { list, install, uninstall, upgrade } from '../lib/plugins'

describe('plugins', () => {
  test('list', () => {
    expect(list()).toEqual(expect.arrayContaining([]))
  })

  test('install', () => {
    expect(install('javascript')).toEqual(expect.arrayContaining([]))
  })

  test('uninstall', () => {
    expect(uninstall('javascript')).toEqual(expect.arrayContaining([]))
  })

  test('upgrade', () => {
    expect(upgrade('javascript')).toEqual(expect.arrayContaining([]))
  })
})
