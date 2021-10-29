/**
 * Test that custom selectors used in theme CSS are valid
 * (i.e. that they exist in `src/selectors.css`
 */
import fs from 'fs'
import globby from 'globby'
import path from 'path'
import { promisify } from 'util'

describe('semantic selectors are valid', () => {
  const srcDir = path.join(__dirname, '..', 'src')

  const selectors = fs
    .readFileSync(path.join(srcDir, 'selectors.css'), 'utf8')
    .split('\n')
    .filter((line) => line.startsWith('@custom-selector'))
    .map((line) => /(:--\w+)/.exec(line)?.[0])

  const themesDir = path.join(srcDir, 'themes')
  const files = globby.sync('**/*.css', {
    cwd: themesDir,
  })
  test.each(files)('%s', async (theme) => {
    const styles = await promisify(fs.readFile)(
      path.join(themesDir, theme),
      'utf8'
    )
    const matches = styles.match(/:--\w+/g) ?? []
    const invalid = matches.reduce(
      (prev: string[], selector) =>
        !selectors.includes(selector) && !prev.includes(selector)
          ? [...prev, selector]
          : prev,
      []
    )
    expect(invalid).toEqual([])
  })
})
