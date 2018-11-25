import {default as export_, exportJsonLd} from '../src/export'
import SoftwarePackage from '../src/SoftwarePackage'
import Thing from '../src/Thing'

test('export:Thing', () => {
  const thing = new Thing()
  expect(export_(thing)).toEqual(exportJsonLd(thing))
  expect(export_(exportJsonLd(thing), 'application/ld+json')).toEqual(exportJsonLd(thing))
  expect(() => export_(thing, 'foo/bar')).toThrow(/^Unhandled export format: foo\/bar/)
})

test('exportJsonLd:Thing', () => {
  const thing1 = new Thing()
  expect(JSON.parse(exportJsonLd(thing1))).toEqual({
    '@context': 'https://stencila.github.io/schema/context.jsonld',
    'type': 'Thing'
  })

  const thing2 = new Thing({
    name: 'thing2',
    description: 'The second thing',
    identifiers: ['thing2', 'thing_two'],
    urls: []
  })
  expect(JSON.parse(exportJsonLd(thing2))).toEqual({
    '@context': 'https://stencila.github.io/schema/context.jsonld',
    'type': 'Thing',
    'name': 'thing2',
    'description': 'The second thing',
    'identifier': ['thing2', 'thing_two']
  })
})

test('exportJsonLd:SoftwarePackage', () => {
  const pkg = new SoftwarePackage()
  pkg.name = 'My package'
  pkg.softwareRequirements = [
    new SoftwarePackage({ name: 'another'}),
    new SoftwarePackage({ name: 'yetAnother'})
  ]
  expect(JSON.parse(exportJsonLd(pkg))).toEqual({
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
