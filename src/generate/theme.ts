/**
 * A script to create a new theme folder in `../themes/`.
 *
 * Run using `npm run create:theme -- <name-of-theme>`.
 *
 * Builds the
 */

import fs from 'fs'
import path from 'path'
import { generateThemes } from './themes'

if (module.parent === null) addTheme(process.argv[2])

export function addTheme(name?: string): void {
  // Check that a name has been supplied
  if (name === undefined) {
    console.log(`You must supply a theme name`)
    process.exit(1)
  }

  const themeDir = path.join(__dirname, '..', 'themes', name)

  // Check that the theme does not already exist
  if (fs.existsSync(themeDir)) {
    console.log(`Theme "${name}" already exists: ${themeDir}`)
    process.exit(1)
  } else {
    fs.mkdirSync(themeDir)
  }

  // Create necessary files
  fs.writeFileSync(
    path.join(themeDir, 'README.md'),
    `# ${name[0].toUpperCase()}${name.slice(1)}

<!-- Add a description of your theme and notes for contributors. -->\n`
  )

  fs.writeFileSync(
    path.join(themeDir, 'index.ts'),
    `export function init() {
  // Do any DOM manipulation that your theme needs here
}\n`
  )

  fs.writeFileSync(
    path.join(themeDir, 'styles.css'),
    `/* Add your theme's styles to this file */\n`
  )

  // Update `themes.ts` etc
  generateThemes()
}
