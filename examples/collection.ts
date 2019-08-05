import { Collection } from '../ts/types'
import * as article from './article'
import * as datatable from './datatable'

export const minimal: Collection = {
  type: 'Collection',
  parts: []
}

export const simple: Collection = {
  type: 'Collection',
  editors: [
    {
      type: 'Person'
    }
  ],
  publisher: {
    type: 'Organization'
  },
  parts: [article.simple, datatable.minimal]
}
