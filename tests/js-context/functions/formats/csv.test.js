import test from 'tape'

import csv from '../../../../src/js-context/functions/formats/csv'

test('csv', t => {
  t.deepEqual(
    csv('x,y\n1,2\n3,4'), 
    { type: 'table', data: { x: { values: [ 1, 3 ] }, y: { values: [ 2, 4 ] } }}
  )

  t.end()
})
