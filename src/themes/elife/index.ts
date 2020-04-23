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

const getReferences = (): Element[] => {
  return select(':--reference')
}

const moveReferenceTitles = (references: Element[]): void => {
  references.forEach((reference: Element) => {
    const headline: Element | null = first(reference, ':--title')
    if (headline !== null) {
      prepend(reference, headline)
    }
  })
}

const moveVolumeNumber = (references: Element[]): void => {
  references.forEach((reference: Element) => {
    const volumeNumber: Element | null = first(reference, ':--volumeNumber')
    if (volumeNumber !== null) {
      append(reference, volumeNumber)
    }
  })
}

const movePageStart = (references: Element[]): void => {
  references.forEach((reference: Element) => {
    const pageStart: Element | null = first(reference, ':--pageStart')
    if (pageStart !== null) {
      append(reference, pageStart)
    }
  })
}

const movePageEnd = (references: Element[]): void => {
  references.forEach((reference: Element) => {
    const pageEnd: Element | null = first(reference, ':--pageEnd')
    if (pageEnd !== null) {
      append(reference, pageEnd)
    }
  })
}

ready((): void => {
  formatDate(first(':--datePublished'))

  downloads.build(
    getArticleId(),
    first(':--title')?.getAttribute('content') ?? ''
  )

  const references = getReferences()
  moveReferenceTitles(references)
  moveVolumeNumber(references)
  movePageStart(references)
  movePageEnd(references)
})
