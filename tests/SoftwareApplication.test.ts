import SoftwareApplication from '../src/SoftwareApplication'

describe('SoftwareApplication', () => {
  const node = new SoftwareApplication()

  test('type', () => {
    expect(node.type).toEqual('SoftwareApplication')
  })

})
