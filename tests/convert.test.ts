import convert from '../src/convert'

test('convert:Thing', () => {
  const jsonld = '{"@context":"https://stencila.github.io/schema/context.jsonld","type":"Thing","name":"Thing1"}'
  expect(convert(jsonld, 'application/ld+json', 'application/ld+json')).toEqual(jsonld)
  expect(() => convert('blah', 'foo/bar')).toThrow(/^Unhandled import format: foo\/bar/)
  expect(() => convert('{"type":"Person"}', undefined, 'foo/bar')).toThrow(/^Unhandled export format: foo\/bar/)
})
