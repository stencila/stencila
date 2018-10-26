import CreativeWork from '../src/CreativeWork'

describe('CreativeWork', () => {
  const node = new CreativeWork()

  test('type', () => {
    expect(node.type).toEqual('CreativeWork')
  })

})
