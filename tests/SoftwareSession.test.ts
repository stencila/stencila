import SoftwareSession from '../src/SoftwareSession'

describe('SoftwareSession', () => {
  const node = new SoftwareSession()

  test('type', () => {
    expect(node.type).toEqual('SoftwareSession')
  })

})
