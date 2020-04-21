import pkg from '../../package.json'
import {
  jsonLdUrl,
  jsonLdContext,
  jsonLdTermUrl,
  jsonLdTermName,
} from './jsonld'

test('jsonLdUrl', () => {
  const expectedBase = `http://schema.stenci.la/v${
    pkg.version.split('.')[0]
  }/jsonld/`

  expect(jsonLdUrl()).toMatch(expectedBase)
  expect(jsonLdUrl('CodeChunk')).toMatch(expectedBase + 'CodeChunk')
  expect(jsonLdUrl('outputs')).toMatch(expectedBase + 'outputs')
})

test('jsonLdContext', () => {
  expect(jsonLdContext()).toHaveProperty('stencila')
  expect(jsonLdContext().stencila).toEqual(jsonLdUrl())
})

test('jsonLdTermUrl', () => {
  const stencilaUrl = jsonLdUrl()
  expect(jsonLdTermUrl('CodeChunk')).toEqual(stencilaUrl + 'CodeChunk')
  expect(jsonLdTermUrl('outputs')).toEqual(stencilaUrl + 'outputs')

  expect(jsonLdTermUrl('Article')).toEqual('http://schema.org/Article')
  expect(jsonLdTermUrl('authors')).toEqual('http://schema.org/author')

  expect(jsonLdTermUrl('maintainers')).toEqual(
    'http://doi.org/10.5063/schema/codemeta-2.0#maintainer'
  )

  expect(jsonLdTermUrl('foo')).toBeUndefined()
})

test('jsonLdTermName', () => {
  const stencilaUrl = jsonLdUrl()
  expect(jsonLdTermName(stencilaUrl + 'CodeChunk')).toEqual('CodeChunk')
  expect(jsonLdTermName(stencilaUrl + 'outputs')).toEqual('outputs')

  expect(jsonLdTermName('http://schema.org/Article')).toEqual('Article')
  expect(jsonLdTermName('http://schema.org/author')).toEqual('authors')

  expect(
    jsonLdTermName('http://doi.org/10.5063/schema/codemeta-2.0#maintainer')
  ).toEqual('maintainers')
})
