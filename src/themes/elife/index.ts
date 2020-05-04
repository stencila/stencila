import { first, ready, select, text } from '../../util'
import * as dateFormatter from './lib/dateFormatter'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as references from './lib/references'

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
  dateFormatter.format(first(':--datePublished'))

  downloads.build(
    getArticleId(),
    first(':--title')?.getAttribute('content') ?? ''
  )

  references.transform(select(':--reference'))

  try {
    socialSharers.build(getArticleTitle(), getArticleDoi())
  } catch (e) {
    console.error(e)
  }
})
