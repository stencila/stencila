export const parseQueries = (themeKeys: string[]): Record<string, string> => {
  const queries = window.location.search
  return queries
    .replace('?', '')
    .split('&')
    .reduce((qs: Record<string, string>, q) => {
      const [key, value] = q.split('=')
      return themeKeys.includes(key)
        ? { ...qs, [key]: decodeURIComponent(value) }
        : qs
    }, {})
}

const regex = (query: string): RegExp => new RegExp(`(&?${query})=([^&]+)`)

export const upsertQuery = (query: string, value: string): string => {
  const url = new URL(window.location.href)
  const queries = decodeURIComponent(url.search)

  if (queries.includes(query)) {
    url.search = queries.replace(regex(query), `$1=${value}`)
    history.pushState(null, 'none', url.toString())
    return url.toString()
  }

  const q = queries.startsWith('?')
    ? `&${encodeURIComponent(query)}=${encodeURIComponent(value)}`
    : `?${encodeURIComponent(query)}=${encodeURIComponent(value)}`

  url.search = queries + q

  history.pushState(null, 'none', url.toString())
  return url.toString()
}

export const removeQuery = (query: string): void => {
  const url = window.location.href.replace(regex(query), '')
  history.pushState(null, 'none', url)
}
