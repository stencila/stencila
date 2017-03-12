import test from 'tape'

import {table1} from '../fixtures'
import bars from '../../../src/functions/plot/bars'

test('bars', t => {
  t.deepEqual(bars(table1), {
    type: 'vegalite',
    data: {
      values: table1
    },
    mark: 'bar',
    encoding: {
      x: {
        field: 'x',
        type: 'qualitative'
      },
      y: {
        field: 'y',
        type: 'quantitative'
      }
    }
  })

  t.end()
})
