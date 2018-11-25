import SoftwareApplication from '../../src/types/SoftwareApplication'

describe('SoftwareApplication', () => {
  const node = new SoftwareApplication()

  test('type', () => {
    expect(node.type).toEqual('SoftwareApplication')
  })

})
