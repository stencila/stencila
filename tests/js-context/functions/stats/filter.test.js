import test from 'tape'

import filter from '../../../../src/js-context/functions/stats/filter'
import {table1} from '../fixtures.js'

test('filter', t => {
  t.deepEqual(filter(table1, 'x<=11'), {
    type: 'tab',
    data: {
      x: [10, 11],
      y: [20, 21]
    }
  })
  t.end()
})
