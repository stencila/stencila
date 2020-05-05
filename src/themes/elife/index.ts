import { first, ready, select } from '../../util'
import * as dateFormatter from './lib/dateFormatter'
import * as dataProvider from './lib/dataProvider'
import * as downloads from './lib/downloads'
import * as socialSharers from './lib/socialSharers'
import * as references from './lib/references'

ready((): void => {
  dateFormatter.format(first(':--datePublished'))

  downloads.build(
    dataProvider.getArticleId(),
    first(':--title')?.getAttribute('content') ?? ''
  )

  references.transform(select(':--reference'))

  try {
    socialSharers.build(
      dataProvider.getArticleTitle(),
      dataProvider.getArticleDoi()
    )
  } catch (e) {
    console.error(e)
  }
})
