import * as referencesFormatter from '../lib/referencesFormatter'
import { getFixtureData } from './fixtures/referencesData.fixture'
import { select, translate } from '../../../util'

const getFirst = (references: Element[]): Element => {
  const firstReference = references[0]
  if (firstReference === null || firstReference === undefined) {
    throw new Error('No reference found in fixture data')
  }
  return firstReference
}

const getElement = (searchRoot: Element, customSelector: string): Element => {
  const element = searchRoot.querySelector(translate(customSelector))
  if (element === null) {
    throw new Error(
      `No elements found matching the semantic selector ${customSelector}`
    )
  }
  return element
}

describe('Formatting a reference', () => {
  let references: Element[]
  let firstReference: Element

  beforeEach(() => {
    references = select(getFixtureData(), ':--reference')
    firstReference = getFirst(references)
  })

  it('the title is the second element', () => {
    referencesFormatter.format(references)
    expect(
      getElement(firstReference, ':--title').isSameNode(
        firstReference.children[2]
      )
    ).toBe(true)
  })

  it('the title follow the authors', () => {
    referencesFormatter.format(references)

    expect(
      getElement(firstReference, ':--authors').isSameNode(
        firstReference.children[0]
      )
    ).toBe(true)
  })
  it('the publication year follows the authors', () => {
    referencesFormatter.format(references)

    expect(
      getElement(firstReference, ':--datePublished').isSameNode(
        firstReference.children[1]
      )
    ).toBe(true)
  })

  describe('the publication volume', () => {
    let volume: Element

    beforeEach(() => {
      referencesFormatter.format(references)
      volume = getElement(firstReference, ':--PublicationVolume:--isPartOf')
    })

    it('follows the publication year', () => {
      expect(volume.isSameNode(firstReference.children[3])).toBe(true)
    })

    it('its volume name is first', () => {
      const volumeName: Element = getElement(volume, ':--isPartOf')
      expect(volumeName.isSameNode(volume.children[0])).toBe(true)
    })

    it('its volume number is second', () => {
      const volumeNumber: Element = getElement(volume, ':--volumeNumber')
      expect(volumeNumber.isSameNode(volume.children[1])).toBe(true)
    })
  })
})
