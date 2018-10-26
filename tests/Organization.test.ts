import Organization from '../src/Organization'

describe('Organization', () => {
  const node = new Organization()

  test('type', () => {
    expect(node.type).toEqual('Organization')
  })

})
