import Organization from '../src/types/Organization'

describe('Organization', () => {
  const node = new Organization()

  test('type', () => {
    expect(node.type).toEqual('Organization')
  })

})
