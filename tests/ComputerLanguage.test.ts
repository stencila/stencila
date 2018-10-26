import ComputerLanguage from '../src/ComputerLanguage'

describe('ComputerLanguage', () => {
  const node = new ComputerLanguage()

  test('type', () => {
    expect(node.type).toEqual('ComputerLanguage')
  })

})
