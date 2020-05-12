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

const moveVolumeNames = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const volume = first(reference, ':--PublicationVolume:--isPartOf')
    if (volume !== null) {
      const volumeName = first(volume, ':--isPartOf')
      if (volumeName !== null) {
        prepend(volume, volumeName)
      }
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

export const format = (references: Element[]): void => {
  movePagesEnd(movePagesStart(moveVolumeNames(moveTitles(references))))
}
