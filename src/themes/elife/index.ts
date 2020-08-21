import { first, ready, select } from '../../util'
import * as contentHeader from './lib/contentHeader'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as icons from './lib/icons'
import * as socialSharers from './lib/socialSharers'
import * as referenceFormatter from './lib/referencesFormatter'
import query from './lib/query'

ready((): void => {
  const articleId = dataProvider.getArticleId()
  const articleTitle = dataProvider.getArticleTitle()
  const contentHeaderElement = contentHeader.build() as Element
  query(articleId, window.fetch)
    .then((response) => {
      icons.build(contentHeaderElement, response.article)
      downloads.build(
        contentHeaderElement,
        articleTitle,
        articleId,
        response.article
      )
    })
    .catch((e) => {
      console.log(e)
    })

  try {
    dateFormatter.format(first(':--datePublished'))
    socialSharers.build(articleTitle, articleId)
    referenceFormatter.format(select(':--reference'))
  } catch (e) {
    console.error(e)
  }
})

// Use solid icons for Stencila components
document
  .getElementsByTagName('html')[0]
  ?.setAttribute('data-icon-style', 'fill')
