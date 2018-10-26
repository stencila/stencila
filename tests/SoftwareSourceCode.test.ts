import SoftwareSourceCode from '../src/SoftwareSourceCode'

describe('SoftwareSourceCode', () => {
  const node = new SoftwareSourceCode()

  test('type', () => {
    expect(node.type).toEqual('SoftwareSourceCode')
  })

})
