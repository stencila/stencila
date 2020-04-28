import { first, ready, prepend, append, select } from '../../util'
import * as downloads from './downloads'
import DateTimeFormat = Intl.DateTimeFormat

const dateFormatter = new DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  year: 'numeric',
})

const formatDate = (dateEl: Element | null): void => {
  if (dateEl instanceof Element) {
    const date = new Date(dateEl.innerHTML)
    dateEl.innerHTML = dateFormatter.format(date)
  }
}

const getArticleId = (): string => {
  const selector =
    ':--identifier meta[content="https://registry.identifiers.org/registry/publisher-id"] ~ [itemprop="value"]'
  return first(selector)?.innerHTML ?? ''
}

const moveReferenceTitles = (references: Element[]): Element[] => {
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

ready((): void => {
  formatDate(first(':--datePublished'))

  downloads.build(
    getArticleId(),
    first(':--title')?.getAttribute('content') ?? ''
  )

  movePagesEnd(
    movePagesStart(
      movePeriodicalNames(
        moveVolumeNumbers(moveReferenceTitles(select(':--reference')))
      )
    )
  )
})
