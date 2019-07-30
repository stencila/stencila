/**
 * Tests of the generation of Typescript language bindings
 *
 * This test suite uses file snapshots. During development
 * it can be useful to update the snapshots on the fly for manual inspection:
 *
 * ```bash
 * npx jest typescript.test.ts --watch --updateSnapshot
 * ```
 */

import * as typescript from 'typescript'
import { build, typeGenerator, unionGenerator } from '../ts/bindings/typescript'
import { schema, snapshot } from './helpers'

test('generators', async () => {
  expect(await typeGenerator(await schema('Thing.schema.json'))).toMatchFile(
    snapshot('Thing.ts')
  )
  expect(await typeGenerator(await schema('Person.schema.json'))).toMatchFile(
    snapshot('Person.ts')
  )

  expect(
    await unionGenerator(await schema('BlockContent.schema.json'))
  ).toMatchFile(snapshot('BlockContent.ts'))
})

/**
 * Build `dist/types.ts` and compile it using Typescript to
 * check that there are no errors.
 */
test('build', async () => {
  const file = await build()
  const program = typescript.createProgram([file], {})
  const diagnostics = typescript
    .getPreEmitDiagnostics(program)
    .map(diagnostic => {
      return diagnostic.messageText
    })
  expect(diagnostics).toEqual([])
})
