import ComputerLanguage from '../src/types/ComputerLanguage'

describe('ComputerLanguage', () => {
  const node = new ComputerLanguage()

  test('type', () => {
    expect(node.type).toEqual('ComputerLanguage')
  })

})
