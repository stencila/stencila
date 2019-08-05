/**
 * Tests of the generation of R language bindings
 *
 * This test suite uses file snapshots. During development
 * it can be useful to update the snapshots on the fly for manual inspection:
 *
 * ```bash
 * npx jest r.test.ts --watch --updateSnapshot
 * ```
 */

import { classGenerator, enumToType, unionGenerator } from './r'
import { schema, snapshot } from '../__tests__/helpers'

test('generators', async () => {
  expect(await classGenerator(await schema('Person.schema.json'))).toMatchFile(
    snapshot(__dirname, 'Person.R')
  )

  expect(
    await unionGenerator(await schema('BlockContent.schema.json'))
  ).toMatchFile(snapshot(__dirname, 'BlockContent.R'))

  const list = await schema('List.schema.json')
  expect(await enumToType(list.properties.order.enum)).toEqual(
    'Enum("ascending", "descending", "unordered")'
  )
})
