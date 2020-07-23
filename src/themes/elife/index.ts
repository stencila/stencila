import { first, ready, select } from '../../util'
import * as contentHeader from './lib/contentHeader'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as icons from './lib/icons'
import * as socialSharers from './lib/socialSharers'
import * as referenceFormatter from './lib/referencesFormatter'

ready((): void => {
  const articleTitle = dataProvider.getArticleTitle()
  const contentHeaderElement = contentHeader.build() as Element
  icons.build(contentHeaderElement, dataProvider.getArticleId())
  downloads.build(
    contentHeaderElement,
    articleTitle,
    dataProvider.getArticleId()
  )

  try {
    dateFormatter.format(first(':--datePublished'))
    socialSharers.build(articleTitle, dataProvider.getArticleDoi())
    referenceFormatter.format(select(':--reference'))
  } catch (e) {
    console.error(e)
  }
})
