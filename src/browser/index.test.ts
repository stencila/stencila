import { themes } from '../themes/index'
import { isTheme } from '.'

describe('Check if `theme` is a Thema theme', () => {
  test.each(Object.keys(themes))('Thema themes - %s', themeKey => {
    expect(isTheme(themeKey)).toBe(true)
  })

  test('File paths', () => {
    expect(isTheme('/my/path/to/directory')).toBe(false)
  })

  test('URLs', () => {
    expect(isTheme('http://myCustomTheme.com')).toBe(false)
  })
})
