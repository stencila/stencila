import Intangible from '../src/Intangible'

describe('Intangible', () => {
  const node = new Intangible()

  test('type', () => {
    expect(node.type).toEqual('Intangible')
  })

})
