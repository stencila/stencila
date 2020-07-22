import { first, ready, select } from '../../util'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as preHeader from './lib/preHeader'
import * as referenceFormatter from './lib/referencesFormatter'

ready((): void => {
  const preHeaderWrapper = preHeader.build() as Element
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
