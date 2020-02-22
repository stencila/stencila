import pkg from '../../package.json'
import { jsonLdUrl } from './urls'

test('jsonLdUrl', () => {
  const expectedBase = `http://schema.stenci.la/v${
    pkg.version.split('.')[0]
  }/jsonld/`
  expect(jsonLdUrl()).toMatch(expectedBase)
  expect(jsonLdUrl('CodeChunk')).toMatch(expectedBase + 'CodeChunk')
  expect(jsonLdUrl('outputs')).toMatch(expectedBase + 'outputs')
})
