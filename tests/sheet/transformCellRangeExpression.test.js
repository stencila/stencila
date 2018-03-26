import test from 'tape'

import { transformCellRangeExpression } from '../../src/shared/cellHelpers'

const MSG = 'expression should be transformed correctly'
/*
Test cases: (for rows and  columns)

ranges:
- range expression inside deleted
- deleted range inside of range expression
- range expression overlaps deleted range on the 'left' side
- range expression overlaps deleted range on the 'right' sides

single:
- before, inside, after

*/

test('transformCellRangeExpression: single / col / insert before', (t) => {
  let expr = 'C1'
  let mode = 'col'
  let idx = 1
  let count = 1
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'D1', MSG)
  t.end()
})