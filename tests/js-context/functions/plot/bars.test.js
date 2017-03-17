import test from 'tape'

import {table1} from '../fixtures'
import bars from '../../../../src/js-context/functions/plot/bars'
import convertTableToArray from '../../../../src/js-context/functions/types/convertTableToArray'


test('bars', t => {
  t.deepEqual(bars(table1), {
    type: 'vegalite',
    data: {
      values: convertTableToArray(table1)
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
