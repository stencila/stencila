import DateTimeFormat = Intl.DateTimeFormat

const formatter = new DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  year: 'numeric',
})

export const format = (dateEl: Element | null): void => {
  if (dateEl instanceof Element) {
    dateEl.innerHTML = formatter.format(new Date(dateEl.innerHTML))
  }
}
