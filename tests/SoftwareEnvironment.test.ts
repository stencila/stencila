import SoftwareEnvironment from '../src/SoftwareEnvironment'

describe('SoftwareEnvironment', () => {
  const node = new SoftwareEnvironment()

  test('type', () => {
    expect(node.type).toEqual('SoftwareEnvironment')
  })

})
