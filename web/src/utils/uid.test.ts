import { generate } from './uid'

describe('generate', () => {
  test('structure', () => {
    expect(generate('abc').length).toBe(23)
    expect(generate('a')).toMatch(/^aa-([0-9A-Za-z]{20})$/)
    expect(generate('cc')).toMatch(/^cc-([0-9A-Za-z]{20})$/)
    expect(generate('code')).toMatch(/^co-([0-9A-Za-z]{20})$/)
  })

  test('size', () => {
    expect(generate('ab').length).toBe(23)
    expect(generate('ab', 30).length).toBe(33)
    expect(generate('code', 50)).toMatch(/^co-([0-9A-Za-z]{50})$/)
  })
})
