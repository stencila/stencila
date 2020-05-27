/**
 * A script to update `../README.md`
 *
 * Run using `npm run docs:readme`.
 */

import { dump, read } from '@stencila/encoda'
import {
  Article,
  link,
  Paragraph,
  table,
  tableCell,
  tableRow,
} from '@stencila/schema'
import fs from 'fs'
import jsdoc2md from 'jsdoc-to-markdown'
import path from 'path'
import typescript from 'typescript'
import { promisify } from 'util'
import { extensions } from '../extensions'
import { themes } from '../themes/index'

const readFile = promisify(fs.readFile)
const writeFile = promisify(fs.writeFile)

if (module.parent === null) update()

function update(): void {
  const readmePath = path.join(__dirname, '..', '..', 'README.md')
  readFile(readmePath, 'utf8')
    .then(generateThemeDocs)
    .then(generateExtsDocs)
    .then(generateApiDocs)
    .then((readme) => writeFile(readmePath, readme))
    .catch(console.error)
}

/**
 * Update docs on available themes
 *
 * @param {string} readme The contents of the README
 * @returns {Promise<string>}
 */ async function generateThemeDocs(readme: string): Promise<string> {
  const md = await readmesToTable(
    path.join(__dirname, '..', 'themes'),
    Object.keys(themes)
  )
  return readme.replace(
    /<!-- THEMES-START -->[\s\S]*?<!-- THEMES-END -->/gm,
    `<!-- THEMES-START -->\n\n${md}\n\n<!-- THEMES-END -->`
  )
}

/**
 * Update docs on available extensions
 *
 * @param {string} readme The contents of the README
 * @returns {Promise<string>}
 */
async function generateExtsDocs(readme: string): Promise<string> {
  const md = await readmesToTable(
    path.join(__dirname, '..', 'extensions'),
    Object.keys(extensions)
  )
  return readme.replace(
    /<!-- EXTS-START -->[\s\S]*?<!-- EXTS-END -->/gm,
    `<!-- EXTS-START -->\n\n${md}\n\n<!-- EXTS-END -->`
  )
}

/**
 * Generate a Markdown table from the READMEs within subdiretories of a directory
 *
 * Currently, uses the first paragraph as the description of each sub-directory.
 *
 * @param {string} dir The root directory
 * @param {string[]} subdirs The subdirectories to collect READMEs from
 */
async function readmesToTable(dir: string, subdirs: string[]): Promise<string> {
  const rows = await Promise.all(
    subdirs.map(async (theme) => {
      const readme = (await read(path.join(dir, theme, 'README.md'))) as Article
      const firstParaContent = (readme.content?.[0] as Paragraph)?.content ?? []
      return tableRow({
        cells: [
          tableCell({
            content: [link({ target: `./themes/${theme}`, content: [theme] })],
          }),
          tableCell({ content: firstParaContent }),
        ],
      })
    })
  )
  const tab = table({
    rows: [
      tableRow({
        cells: [
          tableCell({ content: ['Name'] }),
          tableCell({ content: ['Description'] }),
        ],
      }),
      ...rows,
    ],
  })
  return dump(tab, 'md')
}

/**
 * Update API docs from Typescript sources
 *
 * @param {string} readme The contents of the README
 * @returns {Promise<string>}
 */
async function generateApiDocs(readme: string): Promise<string> {
  const ts = await readFile(
    path.join(__dirname, '..', 'util', 'index.ts'),
    'utf8'
  )
  const js = typescript.transpileModule(ts, {}).outputText
  const md = await jsdoc2md.render({
    source: js,
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-expect-error
    'heading-depth': 3,
  })
  return readme.replace(
    /<!-- API-START -->[\s\S]*?<!-- API-END -->/gm,
    `<!-- API-START -->\n${md}\n<!-- API-END -->`
  )
}
