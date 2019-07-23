/**
 * Tests of the generation of Python language bindings
 *
 * This test suite uses file snapshots. During development
 * it can be useful to update the snapshots on the fly for manual inspection:
 *
 * ```bash
 * npx jest python.test.ts --watch --updateSnapshot
 * ```
 */

import { classGenerator, unionGenerator } from '../src/python'
import { schema, snapshot } from './helpers'

test('generators', async () => {
  expect(await classGenerator(await schema('Person.schema.json'))).toMatchFile(
    snapshot('Person.py')
  )

  expect(
    await unionGenerator(await schema('BlockContent.schema.json'))
  ).toMatchFile(snapshot('BlockContent.py'))
})
