import { microdataItemtype, microdataItemprop, microdataType } from './microdata'
import { jsonLdUrl } from './jsonld'

test('microdataItemtype', () => {
  expect(microdataItemtype('CodeChunk')).toMatch(jsonLdUrl('CodeChunk'))
  expect(microdataItemtype('Article')).toMatch('http://schema.org/Article')
  // @ts-ignore that Foo is not a type
  expect(microdataItemtype('Foo')).toBeUndefined()
})

test('microdataType', () => {
  expect(microdataType(jsonLdUrl('CodeChunk'))).toMatch('CodeChunk')
  expect('http://schema.org/Article').toMatch('Article')
  expect(microdataType('http://example.com')).toBeUndefined()
})

test('microdataItemprop', () => {
  expect(microdataItemprop('outputs')).toEqual(['stencila', 'outputs'])

  expect(microdataItemprop('authors')).toEqual(['schema', 'author'])
  expect(microdataItemprop('references')).toEqual(['schema', 'citation'])

  expect(microdataItemprop('maintainers')).toEqual(['codemeta', 'maintainer'])

  expect(microdataItemprop('foo')).toEqual([undefined, undefined])
})
