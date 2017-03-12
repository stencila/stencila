import test from 'tape'

import csv from '../../../src/functions/formats/csv'

test('csv', t => {
  t.deepEqual(csv('x,y\n1,2\n3,4'), [
    {x: 1, y: 2},
    {x: 3, y: 4}
  ])

  t.end()
})
