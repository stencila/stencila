import { before, create, first, ready, select } from '../../util'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as referenceFormatter from './lib/referencesFormatter'

ready((): void | unknown => {
  const articleTitleElement = first(':--Article > :--title')
  if (articleTitleElement === null) {
    return Promise.reject(
      new Error("Can't find element to bolt the pre-header-wrapper on top of")
    )
  }
  const preHeaderWrapper = create('div', { class: 'pre-header-wrapper' })
  before(articleTitleElement, preHeaderWrapper)
  const articleTitle = dataProvider.getArticleTitle()
  downloads.build(preHeaderWrapper, articleTitle, dataProvider.getArticleId())

  try {
    dateFormatter.format(first(':--datePublished'))
    socialSharers.build(articleTitle, dataProvider.getArticleDoi())
    referenceFormatter.format(select(':--reference'))
  } catch (e) {
    console.error(e)
  }
})
