import SoftwarePackage from '../src/types/SoftwarePackage'

test('type', () => {
  const pkg = new SoftwarePackage()
  expect(pkg.type).toEqual('SoftwarePackage')
})
