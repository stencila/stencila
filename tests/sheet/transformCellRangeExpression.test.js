import test from 'tape'

import { transformCellRangeExpression, BROKEN_REF } from '../../src/shared/cellHelpers'

const MSG = 'expression should be transformed correctly'

/*
TODO

ranges:
- range expression inside deleted
- deleted range inside of range expression
- range expression overlaps deleted range on the 'left' side
- range expression overlaps deleted range on the 'right' sides

*/

// cell reference - insert/delete columns

test('transformCellRangeExpression: single / col / insert before', (t) => {
  let expr = 'C1'
  let dim = 'col'
  let idx = 1
  let count = 2
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'E1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / insert after', (t) => {
  let expr = 'C1'
  let dim = 'col'
  let idx = 4
  let count = 1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete before', (t) => {
  let expr = 'C1'
  let dim = 'col'
  let idx = 0
  let count = -2
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'A1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete after', (t) => {
  let expr = 'C1'
  let dim = 'col'
  let idx = 4
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete same', (t) => {
  let expr = 'C1'
  let dim = 'col'
  let idx = 2
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, BROKEN_REF, MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete surrounding', (t) => {
  let expr = 'C1'
  let dim = 'col'
  let idx = 1
  let count = -4
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, BROKEN_REF, MSG)
  t.end()
})

// cell reference - insert/delete rows

test('transformCellRangeExpression: single / row / insert before', (t) => {
  let expr = 'C3'
  let dim = 'row'
  let idx = 1
  let count = 3
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C6', MSG)
  t.end()
})

test('transformCellRangeExpression: single / row / insert after', (t) => {
  let expr = 'C3'
  let dim = 'row'
  let idx = 4
  let count = 1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3', MSG)
  t.end()
})

test('transformCellRangeExpression: single / row / delete before', (t) => {
  let expr = 'C3'
  let dim = 'row'
  let idx = 0
  let count = -2
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / row / delete after', (t) => {
  let expr = 'C3'
  let dim = 'row'
  let idx = 4
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3', MSG)
  t.end()
})

test('transformCellRangeExpression: single / row / delete same', (t) => {
  let expr = 'C3'
  let dim = 'row'
  let idx = 2
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, BROKEN_REF, MSG)
  t.end()
})
test('transformCellRangeExpression: single / row / delete surrounding', (t) => {
  let expr = 'C3'
  let dim = 'row'
  let idx = 1
  let count = -4
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, BROKEN_REF, MSG)
  t.end()
})

// range - insert/delete cols

test('transformCellRangeExpression: range / col / insert before', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 2
  let count = 2
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'E3:H10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / col / insert inside', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 4
  let count = 3
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:I10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / col / insert after', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 7
  let count = 1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:F10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / col / delete before', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 1
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'B3:E10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / col / delete overlapping start', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 1
  let count = -4
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'B3:B10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / col / delete inside', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 3
  let count = -2
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:D10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / col / delete overlapping end', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 4
  let count = -3
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:D10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / col / delete after', (t) => {
  let expr = 'C3:F10'
  let dim = 'col'
  let idx = 7
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:F10', MSG)
  t.end()
})

// range - insert/delete rows

test('transformCellRangeExpression: range / row / insert before', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 1
  let count = 3
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C6:F13', MSG)
  t.end()
})

test('transformCellRangeExpression: range / row / insert inside', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 6
  let count = 2
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:F12', MSG)
  t.end()
})

test('transformCellRangeExpression: range / row / insert after', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 12
  let count = 1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:F10', MSG)
  t.end()
})

test('transformCellRangeExpression: range / row / delete before', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 1
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C2:F9', MSG)
  t.end()
})

test('transformCellRangeExpression: range / row / delete overlapping start', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 1
  let count = -4
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C2:F6', MSG)
  t.end()
})

test('transformCellRangeExpression: range / row / delete inside', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 5
  let count = -2
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:F8', MSG)
  t.end()
})

test('transformCellRangeExpression: range / row / delete overlapping end', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 7
  let count = -6
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:F7', MSG)
  t.end()
})

test('transformCellRangeExpression: range / row / delete after', (t) => {
  let expr = 'C3:F10'
  let dim = 'row'
  let idx = 12
  let count = -1
  let transformed = transformCellRangeExpression(expr, { dim, idx, count})
  t.equal(transformed, 'C3:F10', MSG)
  t.end()
})
