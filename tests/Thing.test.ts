import Thing from '../src/Thing'

describe('Thing', () => {
  const node = new Thing()

  test('type', () => {
    expect(node.type).toEqual('Thing')
  })

})
