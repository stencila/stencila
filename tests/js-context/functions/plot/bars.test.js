import test from 'tape'

import {table1} from '../fixtures'
import bars from '../../../../src/js-context/functions/plot/bars'
import marks from '../../../../src/js-context/functions/plot/marks'

test('bars', t => {
  t.deepEqual(
    bars(table1), 
    marks(table1, 'bar')
  )

  t.end()
})
