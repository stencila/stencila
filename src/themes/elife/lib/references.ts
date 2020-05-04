import { prepend, append, first } from '../../../util'

const moveTitles = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const headline = first(reference, ':--title')
    if (headline !== null) {
      prepend(reference, headline)
    }
  })
  return references
}

const movePeriodicalNames = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const PeriodicalNames = first(reference, ':--isPartOf:--name')
    if (PeriodicalNames !== null) {
      prepend(reference, PeriodicalNames)
    }
  })
  return references
}

const moveVolumeNumbers = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const volumeNumbers = first(reference, ':--volumeNumber')
    if (volumeNumbers !== null) {
      append(reference, volumeNumbers)
    }
  })
  return references
}

const movePagesStart = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const pagesStart = first(reference, ':--pageStart')
    if (pagesStart !== null) {
      append(reference, pagesStart)
    }
  })
  return references
}

const movePagesEnd = (references: Element[]): void => {
  references.forEach((reference: Element): void => {
    const pagesEnd = first(reference, ':--pageEnd')
    if (pagesEnd !== null) {
      append(reference, pagesEnd)
    }
  })
}

export const transform = (references: Element[]): void => {
  movePagesEnd(
    movePagesStart(
      movePeriodicalNames(moveVolumeNumbers(moveTitles(references)))
    )
  )
}
