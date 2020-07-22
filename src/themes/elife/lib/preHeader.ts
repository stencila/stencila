import { before, create, first } from '../../../util'

export const build = (): Element | Promise<never> => {
  const articleTitle = first(':--Article > :--title')
  if (articleTitle === null) {
    return Promise.reject(
      new Error("Can't find element to bolt the pre-header-wrapper on top of")
    )
  }
  const preHeaderWrapper = create('div', { class: 'pre-header-wrapper' })
  before(articleTitle, preHeaderWrapper)
  return preHeaderWrapper
}
