import { first, ready } from '../../util'
import DateTimeFormat = Intl.DateTimeFormat

function elifeFormatDate(date: Date): string {
  const formatter = new DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  })
  return formatter.format(date)
}

ready((): void => {
  const dateEl = first(':--Date')
  if (!(dateEl instanceof Element)) return
  const date = new Date(dateEl.innerHTML)
  dateEl.innerHTML = elifeFormatDate(date)
})
