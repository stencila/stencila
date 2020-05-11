import { first, ready, select } from '../../util'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as referenceFormatter from './lib/referenceFormatter'

ready((): void => {
  const articleTitle = dataProvider.getArticleTitle()
  downloads.build(articleTitle, dataProvider.getArticleId())

  try {
    dateFormatter.format(first(':--datePublished'))
    socialSharers.build(articleTitle, dataProvider.getArticleDoi())
  } catch (e) {
    console.error(e)
  }

  referenceFormatter.format(select(':--reference'))
})
