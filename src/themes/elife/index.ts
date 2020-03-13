import '../../extensions/headings'
import '../../extensions/person'
import { first, ready } from '../../util'
import * as downloads from './downloads'
import DateTimeFormat = Intl.DateTimeFormat

const dateFormatter = new DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  year: 'numeric'
})

const formatDate = (date: Date): string => {
  return dateFormatter.format(date)
}

ready((): void => {
  const dateEl = first(':--Date')
  if (!(dateEl instanceof Element)) return
  const date = new Date(dateEl.innerHTML)
  dateEl.innerHTML = formatDate(date)

  downloads.build('TheArticleId', 'The Article Title')
})
