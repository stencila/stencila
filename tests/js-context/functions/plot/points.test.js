import test from 'tape'

import {table1} from '../fixtures'
import points from '../../../../src/js-context/functions/plot/points'
import marks from '../../../../src/js-context/functions/plot/marks'

test('points', t => {
  t.deepEqual(
    points(table1), 
    marks(table1, 'point')
  )

  t.end()
})
