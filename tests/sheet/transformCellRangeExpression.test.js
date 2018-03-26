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
  let mode = 'col'
  let idx = 1
  let count = 2
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'E1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / insert after', (t) => {
  let expr = 'C1'
  let mode = 'col'
  let idx = 4
  let count = 1
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete before', (t) => {
  let expr = 'C1'
  let mode = 'col'
  let idx = 0
  let count = -2
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'A1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete after', (t) => {
  let expr = 'C1'
  let mode = 'col'
  let idx = 4
  let count = -1
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete same', (t) => {
  let expr = 'C1'
  let mode = 'col'
  let idx = 2
  let count = -1
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C1', BROKEN_REF)
  t.end()
})

test('transformCellRangeExpression: single / col / delete surrounding', (t) => {
  let expr = 'C1'
  let mode = 'col'
  let idx = 1
  let count = -4
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C1', BROKEN_REF)
  t.end()
})

// cell reference - insert/delete rows


test('transformCellRangeExpression: single / row / insert before', (t) => {
  let expr = 'C3'
  let mode = 'row'
  let idx = 1
  let count = 3
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C6', MSG)
  t.end()
})

test('transformCellRangeExpression: single / row / insert after', (t) => {
  let expr = 'C3'
  let mode = 'row'
  let idx = 4
  let count = 1
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C3', MSG)
  t.end()
})

test('transformCellRangeExpression: single / row / delete before', (t) => {
  let expr = 'C3'
  let mode = 'row'
  let idx = 0
  let count = -2
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C1', MSG)
  t.end()
})

test('transformCellRangeExpression: single / row / delete after', (t) => {
  let expr = 'C3'
  let mode = 'row'
  let idx = 4
  let count = -1
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, 'C3', MSG)
  t.end()
})

test('transformCellRangeExpression: single / col / delete same', (t) => {
  let expr = 'C3'
  let mode = 'row'
  let idx = 2
  let count = -1
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, BROKEN_REF, MSG)
  t.end()
})
test('transformCellRangeExpression: single / row / delete surrounding', (t) => {
  let expr = 'C3'
  let mode = 'row'
  let idx = 1
  let count = -4
  let transformed = transformCellRangeExpression(expr,  { mode, idx, count})
  t.equal(transformed, BROKEN_REF, MSG)
  t.end()
})
