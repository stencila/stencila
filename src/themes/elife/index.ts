import { first, ready, select } from '../../util'
import * as contentHeader from './lib/contentHeader'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as referenceFormatter from './lib/referencesFormatter'
import query from './lib/query'

ready((): void => {
  const articleId = dataProvider.getArticleId()
  query(articleId, window.fetch)
    .then((response) => {
      downloads.build(
        contentHeader.build() as Element,
        articleTitle,
        articleId,
        response.articleData
      )
    })
    .catch((e) => {
      console.log(e)
    })

  const articleTitle = dataProvider.getArticleTitle()
  try {
    dateFormatter.format(first(':--datePublished'))
    socialSharers.build(articleTitle, dataProvider.getArticleDoi())
    referenceFormatter.format(select(':--reference'))
  } catch (e) {
    console.error(e)
  }
})
