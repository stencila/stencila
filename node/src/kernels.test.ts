import { available } from './kernels'

describe('available', () => {
  test('has at least "calc"', () => {
    let kernels = available()
    expect(kernels).toEqual(expect.arrayContaining(['calc']))
  })
})
