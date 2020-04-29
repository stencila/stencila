import { first, ready, select } from '../../util'
import * as downloads from './downloads'
import * as references from './references'
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

const getArticleDoi = (): string => {
  const selector =
    ':--identifier meta[content="https://registry.identifiers.org/registry/doi"] ~ [itemprop="value"]'
  return first(selector)?.innerHTML ?? ''
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
})
