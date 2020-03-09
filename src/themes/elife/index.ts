import { first, ready } from '../../util'
import DateTimeFormat = Intl.DateTimeFormat

function elifeFormatDate(date: Date): string {
  const formatter: DateTimeFormat = new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  })
  return formatter.format(date)
}

ready((): void => {
  const dateEl: Element | null = first(':--Date')
  if (!(dateEl instanceof Element)) return
  const date: Date = new Date(dateEl.innerHTML)
  const formattedDate: string = elifeFormatDate(date)
  dateEl.innerHTML = formattedDate
})
