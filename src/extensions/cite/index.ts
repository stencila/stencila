import { ready, replace, select, tag } from '../../util'

ready(() =>
  select(':--references :--reference').forEach(reference => {
    // Change `authors` property from list to nested spans
    select(reference, 'ol:--authors').forEach(authors => {
      select(authors, 'li:--author').forEach(author =>
        replace(author, tag(author, 'span'))
      )
      return replace(authors, tag(authors, 'span'))
    })

    // If `publisher` is a div make it a span
    select(reference, 'div:--publisher').forEach(elem =>
      replace(elem, tag(elem, 'span'))
    )
  })
)
