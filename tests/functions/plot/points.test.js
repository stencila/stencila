import test from 'tape'

import {table1} from '../fixtures'
import points from '../../../src/functions/plot/points'

test('points', t => {
  t.deepEqual(points(table1), {
    _vegalite: true,
    data: {
      values: table1
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
