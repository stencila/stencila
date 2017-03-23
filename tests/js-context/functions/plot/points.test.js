import test from 'tape'

import {table1} from '../fixtures'
import plot from '../../../../src/js-context/functions/plot/plot'
import points from '../../../../src/js-context/functions/plot/points'

test('plot', t => {
  t.deepEqual(
    plot(table1, 'point', 'x', 'y'), 
    points(table1, 'x', 'y')
  )

  t.end()
})
