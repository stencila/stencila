import typescript from 'typescript'
import { generateTypeDefinitions } from './typescript'

/**
 * Build `dist/types.ts` and compile it using TypeScript to
 * check that there are no errors.
 */
test('build', async () => {
  const file = await generateTypeDefinitions()
  const program = typescript.createProgram([file], {})
  const diagnostics = typescript
    .getPreEmitDiagnostics(program)
    .map((diagnostic) => {
      return diagnostic.messageText
    })
  expect(diagnostics).toEqual([])
})
