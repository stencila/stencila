import SoftwarePackage from '../src/SoftwarePackage'

test('type', () => {
  const pkg = new SoftwarePackage()
  expect(pkg.type).toEqual('SoftwarePackage')
})
