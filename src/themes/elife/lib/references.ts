import { prepend, append, first } from '../../../util'

export const moveTitles = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const headline = first(reference, ':--title')
    if (headline !== null) {
      prepend(reference, headline)
    }
  })
  return references
}

export const movePeriodicalNames = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const PeriodicalNames = first(reference, ':--isPartOf:--name')
    if (PeriodicalNames !== null) {
      prepend(reference, PeriodicalNames)
    }
  })
  return references
}

export const moveVolumeNumbers = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const volumeNumbers = first(reference, ':--volumeNumber')
    if (volumeNumbers !== null) {
      append(reference, volumeNumbers)
    }
  })
  return references
}

export const movePagesStart = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const pagesStart = first(reference, ':--pageStart')
    if (pagesStart !== null) {
      append(reference, pagesStart)
    }
  })
  return references
}

export const movePagesEnd = (references: Element[]): void => {
  references.forEach((reference: Element): void => {
    const pagesEnd = first(reference, ':--pageEnd')
    if (pagesEnd !== null) {
      append(reference, pagesEnd)
    }
  })
}
