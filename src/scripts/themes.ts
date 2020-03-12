import fs from 'fs'
import globby from 'globby'
import path from 'path'

if (module.parent === null) {
  const [func, arg1, arg2] = process.argv.slice(2)
  if (func === 'create') create(arg1)
  else if (func === 'check') process.exit(check(arg1, arg2 === 'fix'))
  else if (func === 'update') update()
  else console.error(`Unrecognised function: ${func}`)
}

/**
 * Create a new theme folder in `../themes/`.
 *
 * Run using `npm run create:theme -- <name-of-theme>`.
 *
 * Creates the folder and populates with the necessary files
 * containing placeholder content.
 */
function create(name?: string): void {
  // Check that a name has been supplied
  if (name === undefined) {
    console.error(`Please supply a theme name`)
    process.exit(1)
  }

  // Check that the theme does not already exist
  const themeDir = path.join(__dirname, '..', 'themes', name)
  if (fs.existsSync(themeDir)) {
    console.log(`Theme "${name}" already exists: ${themeDir}`)
    process.exit(1)
  }

  // Check with `fix: true`, to create necessary files
  check(name, true)

  // Update `themes/index.ts` etc
  update()
}

/**
 * Check that a theme has the necessary files.
 *
 * If the the theme `name` is `*`, will check all themes.
 * Use `fix` to fix errors for some checks.
 *
 * Run using `npm run check:theme -- <theme> [fix]`.
 */
function check(name?: string, fix = false): number {
  const themesDir = path.join(__dirname, '..', 'themes')

  if (name === undefined) {
    console.error(`Please supply a theme name`)
    process.exit(1)
  }
  if (name === '*') {
    return globby
      .sync('*', {
        onlyDirectories: true,
        cwd: themesDir
      })
      .reduce((sum, theme) => sum + check(theme, fix), 0)
  }

  const messages: string[] = []
  const error = (message: string): number => messages.push(message)

  // Check that theme dir exists
  const themeDir = path.join(themesDir, name)
  if (!fs.existsSync(themeDir)) {
    if (fix) fs.mkdirSync(themeDir)
    else {
      error(`Theme directory does not exist: ${themeDir}`)
    }
  }

  // Check that README.md exists
  const readme = path.join(themeDir, 'README.md')
  if (!fs.existsSync(readme)) {
    if (fix) {
      fs.writeFileSync(
        readme,
        `# ${name[0].toUpperCase()}${name.slice(1)}

<!-- Add a description of your theme and notes for contributors. -->\n`
      )
    } else {
      error(`Theme README.md does not exist: ${readme}`)
    }
  }

  // Check that styles.css exists
  const styles = path.join(themeDir, 'styles.css')
  if (!fs.existsSync(styles)) {
    if (fix) {
      fs.writeFileSync(styles, `/* Add your theme's styles to this file */\n`)
    } else {
      error(`Theme styles file does not exist: ${styles}`)
    }
  }

  // Check that index.{ts,js} exists
  const indexGlob = path.join(themeDir, 'index.{ts,js}')
  const indexFiles = globby.sync(indexGlob)
  let indexFile = indexFiles[0]
  if (indexFile === undefined) {
    if (fix) {
      indexFile = path.join(themeDir, 'index.ts')
      fs.writeFileSync(
        indexFile,
        `// Import any extensions and do any DOM manipulation that your theme needs in this file\n`
      )
    } else error(`Theme script file does not exist: ${indexGlob}`)
  } else if (indexFiles.length > 1)
    error(`Theme has more than one script file matching: ${indexGlob}`)

  // Check that any other themes or extensions included in CSS, are also
  // included in scripts
  if (indexFile !== undefined) {
    const script = fs.readFileSync(indexFile, 'utf8')
    const css = fs.readFileSync(styles, 'utf8')
    const imports: string[] = []

    // Base theme
    {
      const cssRegex = /^@import\s+'\.\.\/([\w-]+)\/styles\.css'/gm
      let cssMatch
      let cssMatches = 0
      while ((cssMatch = cssRegex.exec(css)) !== null) {
        cssMatches += 1
        if (cssMatches === 1) {
          const theme = cssMatch[1]
          const scriptRegex = new RegExp(`^import\\s+'\\.\\.\\/${theme}'`, 'm')
          if (!scriptRegex.test(script)) {
            if (fix) imports.push(`import '../${theme}'`)
            else
              error(
                `Theme script file does not import base theme "${theme}": ${indexFile}`
              )
          }
        } else error(`Theme extends more than one base theme: ${styles}`)
      }
    }

    // Extensions
    {
      const cssRegex = /^@import\s+'\.\.\/\.\.\/extensions\/([\w-]+)\/styles\.css'/gm
      let cssMatch
      while ((cssMatch = cssRegex.exec(css)) !== null) {
        const extension = cssMatch[1]
        const scriptRegex = new RegExp(
          `^import\\s+'\\.\\.\\/\\.\\.\\/extensions\\/${extension}'`,
          'm'
        )
        if (
          !scriptRegex.test(script) &&
          globby.sync('index.{js,ts}', {
            cwd: path.join(__dirname, '..', 'extensions', extension)
          }).length > 0
        ) {
          if (fix) imports.push(`import '../../extensions/${extension}'`)
          else
            error(
              `Theme script file does not import extension "${extension}": ${indexFile}`
            )
        }
      }
    }

    if (imports.length > 0) {
      fs.writeFileSync(indexFile, imports.join('\n') + '\n' + script)
    }
  }

  messages.forEach(message => console.error(message))
  return messages.length
}

/**
 * Generate `../themes/index.ts`.
 *
 * Run using `npm run update:themes`.
 *
 * This doesn't actually build any themes, it just checks that
 * they have the necessary files and makes an index of them.
 */
function update(): void {
  const themesDir = path.join(__dirname, '..', 'themes')

  // Get the list of themes
  const themes = globby.sync('*', {
    onlyDirectories: true,
    cwd: themesDir
  })

  // Lint each theme
  themes.forEach(theme => check(theme, true))

  // Write list
  fs.writeFileSync(
    path.join(__dirname, '..', 'themes', 'index.ts'),
    `// Generated by scripts/${path.basename(__filename)}. Do not edit.

/**
 * Map of available theme names
 */
export const themes: {
  ${themes.map(theme => `${theme}: '${theme}'`).join('\n  ')}
} = {
  ${themes.map(theme => `${theme}: '${theme}'`).join(',\n  ')}
}\n`
  )
}
