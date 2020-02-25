/**
 * A script to update API docs for utility functions in `../README.md`
 *
 * Run using `npm run docs:ts`.
 */

import fs from 'fs'
import path from 'path'
/* eslint-disable-next-line */
// @ts-ignore type definitions are available but only for v4
import jsdoc2md from 'jsdoc-to-markdown'
import * as typescript from 'typescript'
import { promisify } from 'util'

const readFile = promisify(fs.readFile)
const writeFile = promisify(fs.writeFile)

/* eslint-disable-next-line */
if (module.parent === null) update()

async function update(): Promise<void> {
  const ts = await readFile(
    path.join(__dirname, '..', 'util', 'index.ts'),
    'utf8'
  )
  const js = typescript.transpileModule(ts, {}).outputText
  const md = await jsdoc2md.render({
    source: js,
    'heading-depth': 3
  })
  const readme = path.join(__dirname, '..', '..', 'README.md')
  const content = await readFile(readme, 'utf8')
  const updated = content.replace(
    /<!-- UTIL-API -->[\s\S]*?<!-- UTIL-API-END -->/gm,
    `<!-- UTIL-API -->\n${md}<!-- UTIL-API-END -->`
  )
  await writeFile(readme, updated)
}
