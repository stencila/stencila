import {jsonSchemas, jsonSchemaTypes, jsonSchemaProperties} from './jsonSchemas'

test('jsonSchemas', async () => {
  const types = await jsonSchemas()
  expect(types).toBeInstanceOf(Object)
  expect(Object.keys(types).length).toBeGreaterThan(0)
})

test('jsonSchemaTypes', async () => {
  const types = await jsonSchemaTypes()
  expect(types).toBeInstanceOf(Array)
  expect(types.length).toBeGreaterThan(0)
  expect(types.includes('Null')).toBeTruthy()
  expect(types.includes('CodeChunk')).toBeTruthy()
  expect(types.includes('Article')).toBeTruthy()
})

test('jsonSchemaProperties', async () => {
  const props = await jsonSchemaProperties()
  expect(props).toBeInstanceOf(Array)
  expect(props.length).toBeGreaterThan(0)
  expect(props.includes('authors')).toBeTruthy()
  expect(props.includes('honorificSuffix')).toBeTruthy()
})
