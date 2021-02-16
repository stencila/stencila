import Ajv from 'ajv'
import betterAjvErrors from 'better-ajv-errors'
import fs from 'fs-extra'
import globby from 'globby'
import path from 'path'
import { build, readSchema } from './schema'

/**
 * Test build function works.
 *
 * Do not cleanup to avoid interfering with
 * other tests that may require the schemas and that
 * may be running in parallel.
 */
test('build', async () => {
  await build(false)
})

/**
 * Check that the `public/*.schema.json` files, generated from `schema/*.schema.yaml` files,
 * are valid JSON Schemas.
 */
test('schemas are valid', async () => {
  const ajv = new Ajv()
  const files = await globby(
    path.join(__dirname, '..', 'public', '*.schema.json')
  )
  for (const file of files) {
    const schema = await fs.readJSON(file)
    if (ajv.validateSchema(schema) !== true) {
      console.log(ajv.errors)
      throw new Error(`💣  Oh, oh, ${file} is invalid`)
    }
  }
})

/**
 * Test inheritance via `extend` keyword
 */
test('inheritance', async () => {
  const thing = await readSchema('Thing')
  const person = await readSchema('Person')

  /* eslint-disable @typescript-eslint/strict-boolean-expressions */

  // All `Thing` properties are in `Person` properties
  expect(
    Object.keys(thing.properties ?? {}).some(
      (name) => !Object.keys(person.properties ?? {}).includes(name)
    )
  ).toBe(false)

  // All `Thing` required properties in `Person` required properties
  expect(
    (thing.required ?? []).some(
      (name: string) => !(person.required ?? []).includes(name)
    )
  ).toBe(false)

  // All `Thing` property aliases in `Person` property aliases
  expect(
    Object.keys(thing.propertyAliases ?? {}).some(
      (name) => !Object.keys(person.propertyAliases ?? {}).includes(name)
    )
  ).toBe(false)
})
