import test from 'tape'

import {table1} from '../fixtures'
import points from '../../../../src/js-context/functions/plot/points'
import convertTableToArray from '../../../../src/js-context/functions/types/convertTableToArray'


test('points', t => {
  t.deepEqual(points(table1), {
    type: 'vegalite',
    data: {
      values: convertTableToArray(table1)
    },
    mark: 'point',
    encoding: {
      x: {
        field: 'x',
        type: 'quantitative'
      },
      y: {
        field: 'y',
        type: 'quantitative'
      }
    }
  })

  t.end()
})
