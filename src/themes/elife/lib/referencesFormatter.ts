import { prepend, first } from '../../../util'

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

export const format = (references: Element[]): void => {
  moveVolumeNames(moveTitles(references))
}
