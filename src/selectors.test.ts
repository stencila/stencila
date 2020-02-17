import { semanticToAttributeSelectors } from './selectors'

describe('semanticToAttributeSelectors', () => {
  const convert = semanticToAttributeSelectors

  it('works for types', () => {
    expect(convert(':--Article')).toEqual(
      "[itemtype~='http://schema.org/Article']"
    )
  })

  it('works for properties', () => {
    expect(convert(':--author')).toEqual(
      "[itemprop~='http://schema.org/author']"
    )
  })

  it('works for compound selectors', () => {
    expect(convert(':--Article :--author')).toEqual(
      "[itemtype~='http://schema.org/Article'] [itemprop~='http://schema.org/author']"
    )
    expect(convert(':--Article > :--author:--Person')).toEqual(
      "[itemtype~='http://schema.org/Article'] > [itemprop~='http://schema.org/author'][itemtype~='http://schema.org/Person']"
    )
  })

  it('works for selectors with no custom selectors', () => {
    expect(convert('')).toEqual('')
    expect(convert('.class')).toEqual('.class')
    expect(convert('#id')).toEqual('#id')
    expect(convert('parent > child')).toEqual('parent > child')
  })
})
