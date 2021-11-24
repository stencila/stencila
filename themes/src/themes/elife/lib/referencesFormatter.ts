import { prepend, first } from '../../../util'

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

const movePeriodical = (references: Element[]): Element[] => {
  references.forEach((reference: Element): void => {
    const volume = first(reference, ':--Periodical:--isPartOf')
    if (volume !== null) {
      const periodicalToMove = first(volume, ':--isPartOf')
      if (periodicalToMove !== null) {
        prepend(volume, periodicalToMove)
      }
    }
  })
  return references
}

export const format = (references: Element[]): void => {
  moveVolumeNames(movePeriodical(references))
}
