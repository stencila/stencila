import { first, ready, select, text } from '../../util'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as references from './lib/references'
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

const normaliseWhitespace = (txt: string): string => {
  return txt.replace(/\n/, ' ').replace(/ \s+|\n+/g, ' ')
}

const getNormalisedTextFromElement = (selector: string): string => {
  const target = first(selector)
  if (target !== null) {
    const sourceText = text(target)
    if (sourceText !== null) {
      return normaliseWhitespace(sourceText)
    }
  }
  return ''
}

const getArticleId = (): string => {
  return getNormalisedTextFromElement(
    ':--identifier meta[content="https://registry.identifiers.org/registry/publisher-id"] ~ [itemprop="value"]'
  )
}

const getArticleDoi = (): string => {
  return getNormalisedTextFromElement(
    ':--identifier meta[content="https://registry.identifiers.org/registry/doi"] ~ [itemprop="value"]'
  )
}

const getArticleTitle = (): string => {
  return getNormalisedTextFromElement(':--title')
}

ready((): void => {
  formatDate(first(':--datePublished'))

  downloads.build(
    getArticleId(),
    first(':--title')?.getAttribute('content') ?? ''
  )

  references.movePagesEnd(
    references.movePagesStart(
      references.movePeriodicalNames(
        references.moveVolumeNumbers(
          references.moveTitles(select(':--reference'))
        )
      )
    )
  )

  try {
    socialSharers.build(getArticleTitle(), getArticleDoi())
  } catch (e) {
    console.error(e)
  }
})
