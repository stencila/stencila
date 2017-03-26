import test from 'tape'

import filter from '../../../../src/js-context/functions/stats/filter'
import table from '../../../../src/js-context/functions/types/table'
import {table1} from '../fixtures.js'

test('filter', t => {
  t.deepEqual(
    filter(table1, 'x<=11'),
    table({
      x: { values: [10, 11] },
      y: { values: [20, 21] } 
    })
  )
  t.end()
})
