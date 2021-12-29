import { languages } from './kernels'

describe('languages', () => {
  test('has at least "Calc"', () => {
    let kernels = languages()
    expect(kernels).toEqual(expect.arrayContaining(['Calc']))
  })
})
