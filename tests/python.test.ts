/**
 * Tests of the generation of Python language bindings
 *
 * This test suite uses fixtures and file snapshots. During development
 * it can be useful to update the snapshots on the fly for manual inspection:
 *
 * ```bash
 * npx jest python.test.ts --watch --updateSnapshot
 * ```
 */

import fs from 'fs-extra'
import path from 'path'
import { classGenerator, unionGenerator } from '../src/python'

const schema = (name: string) =>
  fs.readJSON(path.join(__dirname, '..', 'built', name))
const snapshot = (name: string) =>
  path.join(__dirname, '__file_snapshots__', name)

test('generators', async () => {
  expect(
    await classGenerator(await schema('Person.schema.json'))
  ).toMatchFile(
    snapshot('Person.py')
  )

  expect(
    await unionGenerator(await schema('BlockContent.schema.json'))
  ).toMatchFile(
    snapshot('BlockContent.py')
  )
})
