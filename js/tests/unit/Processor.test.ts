import Processor from '../../src/Processor'
import Person from '../../src/types/Person'
import SoftwarePackage from '../../src/types/SoftwarePackage'
import Thing from '../../src/types/Thing'

const processor = new Processor()

test('manifest', () => {
  const mani = processor.manifest()
  expect(mani.stencila).toBeTruthy()
  expect(mani.services.import).toEqual(['application/ld+json'])
  expect(mani.services.export).toEqual(['application/ld+json'])
  expect(mani.services.compile).toEqual([])
  expect(mani.services.build).toEqual([])
  expect(mani.services.execute).toEqual([])
})

test('import:Thing', () => {
  const thing = new Thing()
  expect(processor.import('{"type": "Thing"}')).toEqual(thing)
  expect(processor.import({type: 'Thing'})).toEqual(thing)
  expect(processor.import(thing)).toEqual(thing)

  expect(processor.import('{"type": "Thing"}', 'application/ld+json')).toEqual(thing)
  expect(() => processor.import("", 'foo/bar')).toThrow(/^Unhandled import format: foo\/bar/)
})

test('import:Person', () => {
  expect(processor.import('{"type": "Person", "name": "Jane"}')).toEqual(new Person({name: "Jane"}))
})

test('export:Thing', () => {
  const thing = new Thing()
  expect(processor.export(thing)).toEqual(processor.exportJsonLd(thing))
  expect(processor.export(processor.exportJsonLd(thing), 'application/ld+json')).toEqual(processor.exportJsonLd(thing))
  expect(() => processor.export(thing, 'foo/bar')).toThrow(/^Unhandled export format: foo\/bar/)
})

test('exportJsonLd:Thing', () => {
  const thing1 = new Thing()
  expect(JSON.parse(processor.exportJsonLd(thing1))).toEqual({
    '@context': 'https://stencila.github.io/schema/context.jsonld',
    'type': 'Thing'
  })

  const thing2 = new Thing({
    name: 'thing2',
    description: 'The second thing',
    identifiers: ['thing2', 'thing_two'],
    urls: []
  })
  expect(JSON.parse(processor.exportJsonLd(thing2))).toEqual({
    '@context': 'https://stencila.github.io/schema/context.jsonld',
    'type': 'Thing',
    'name': 'thing2',
    'description': 'The second thing',
    'identifiers': ['thing2', 'thing_two']
  })
})

test('exportJsonLd:SoftwarePackage', () => {
  const pkg = new SoftwarePackage()
  pkg.name = 'My package'
  pkg.softwareRequirements = [
    new SoftwarePackage({ name: 'another'}),
    new SoftwarePackage({ name: 'yetAnother'})
  ]
  expect(JSON.parse(processor.exportJsonLd(pkg))).toEqual({
    "@context": "https://stencila.github.io/schema/context.jsonld",
    "type": "SoftwarePackage",
    "name": "My package",
    "softwareRequirements": [
      {
        "type": "SoftwarePackage",
        "name": "another"
      }, {
        "type": "SoftwarePackage",
        "name": "yetAnother"
      }
    ]
  })
})

test('convert:Thing', () => {
  const jsonld = '{"@context":"https://stencila.github.io/schema/context.jsonld","type":"Thing","name":"Thing1"}'
  expect(processor.convert(jsonld, 'application/ld+json', 'application/ld+json')).toEqual(jsonld)
  expect(() => processor.convert('blah', 'foo/bar')).toThrow(/^Unhandled import format: foo\/bar/)
  expect(() => processor.convert('{"type":"Person"}', undefined, 'foo/bar')).toThrow(/^Unhandled export format: foo\/bar/)
})

test('compile:Thing', () => {
  const thing = new Thing()
  expect(processor.compile('{"type": "Thing"}')).toEqual(thing)
  expect(processor.compile({type: 'Thing'})).toEqual(thing)
  expect(processor.compile(thing)).toEqual(thing)
})

test('build:Thing', () => {
  const thing = new Thing()
  expect(processor.build('{"type": "Thing"}')).toEqual(thing)
  expect(processor.build({type: 'Thing'})).toEqual(thing)
  expect(processor.build(thing)).toEqual(thing)
})

test('execute:Thing', () => {
  const thing = new Thing()
  expect(processor.execute('{"type": "Thing"}')).toEqual(thing)
  expect(processor.execute({type: 'Thing'})).toEqual(thing)
  expect(processor.execute(thing)).toEqual(thing)
})
