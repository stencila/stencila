import SoftwarePackage from '../src/SoftwarePackage'

test('type', () => {
  const pkg = new SoftwarePackage()
  expect(pkg.type).toEqual('SoftwarePackage')
})

test('toJSON', () => {
  const pkg = new SoftwarePackage()
  pkg.name = 'My package'
  pkg.softwareRequirements = [
    new SoftwarePackage({ name: 'another'}),
    new SoftwarePackage({ name: 'yetAnother'})
  ]
  expect(pkg.toJSONLD()).toEqual({
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
