import { languages } from './kernels'

describe('languages', () => {
  test('has at least "calc"', () => {
    let kernels = languages()
    expect(kernels).toEqual(expect.arrayContaining(['calc']))
  })
})
