import SoftwarePackage from '../src/SoftwarePackage'

describe('SoftwarePackage', () => {
  const node = new SoftwarePackage()

  test('type', () => {
    expect(node.type).toEqual('SoftwarePackage')
  })

})
