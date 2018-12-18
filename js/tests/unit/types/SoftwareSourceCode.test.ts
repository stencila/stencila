import SoftwareSourceCode from '../../../src/types/SoftwareSourceCode'

describe('SoftwareSourceCode', () => {
  const node = new SoftwareSourceCode()

  test('type', () => {
    expect(node.type).toEqual('SoftwareSourceCode')
  })

})
