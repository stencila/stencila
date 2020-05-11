import * as referenceFormatter from '../lib/referenceFormatter'
import { getFixtureData } from './fixtures/referencesData.fixture'
import { select, translate } from '../../../util'

const getFirst = (references: Element[]): Element => {
  const firstReference = references[0]
  if (firstReference === null || firstReference === undefined) {
    throw new Error('No reference found in fixture data')
  }
  return firstReference
}

describe('Formatting a reference', () => {
  let references: Element[]
  let firstReference: Element

  beforeEach(() => {
    references = select(getFixtureData(), ':--reference')
    firstReference = getFirst(references)
  })

  it('the title is the first element', () => {
    const getTitleElement = (reference: Element): Element => {
      const title = reference.querySelector(translate(':--title'))
      if (title === null) {
        throw new Error('No title element found')
      }
      return title
    }

    referenceFormatter.format(Array.from(references))

    expect(
      getTitleElement(firstReference).isSameNode(
        firstReference.firstElementChild
      )
    ).toBe(true)
  })
})
