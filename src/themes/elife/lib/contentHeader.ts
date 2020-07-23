import { before, create, first } from '../../../util'

export const build = (): Element | Promise<never> => {
  const articleTitle = first(':--Article > :--title')
  if (articleTitle === null) {
    return Promise.reject(
      new Error("Can't find element to bolt the content header on top of")
    )
  }
  const contentHeader = create('div', { class: 'content-header' })
  before(articleTitle, contentHeader)
  return contentHeader
}
