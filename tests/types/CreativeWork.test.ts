import CreativeWork from '../src/types/CreativeWork'

describe('CreativeWork', () => {
  const node = new CreativeWork()

  test('type', () => {
    expect(node.type).toEqual('CreativeWork')
  })

})
