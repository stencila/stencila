import SoftwareEnvironment from '../../src/types/SoftwareEnvironment'

describe('SoftwareEnvironment', () => {
  const node = new SoftwareEnvironment()

  test('type', () => {
    expect(node.type).toEqual('SoftwareEnvironment')
  })

})
