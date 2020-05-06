import { first, ready, select } from '../../util'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as references from './lib/references'

ready((): void => {
  dateFormatter.format(first(':--datePublished'))

  const articleTitle = dataProvider.getArticleTitle()
  downloads.build(articleTitle, dataProvider.getArticleId())

  try {
    socialSharers.build(articleTitle, dataProvider.getArticleDoi())
  } catch (e) {
    console.error(e)
  }

  references.transform(select(':--reference'))
})
