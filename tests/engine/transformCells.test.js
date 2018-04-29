import test from 'tape'
import { forEach } from 'substance'
import { recordTransformations, applyCellTransformations, BROKEN_REF } from '../../src/engine/engineHelpers'
import Cell from '../../src/engine/Cell'
import CellSymbol from '../../src/engine/CellSymbol'

const MSG = 'expression should be transformed correctly'

// cell reference - insert/delete columns

test('transformCell: single / col / insert before', (t) => {
  let cell = _createCell('C1')
  let dim = 'col'
  let pos = 1
  let count = 2
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'E1', MSG)
  t.end()
})

test('transformCell: single / col / insert after', (t) => {
  let cell = _createCell('C1')
  let dim = 'col'
  let pos = 4
  let count = 1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C1', MSG)
  t.end()
})

test('transformCell: single / col / delete before', (t) => {
  let cell = _createCell('C1')
  let dim = 'col'
  let pos = 0
  let count = -2
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'A1', MSG)
  t.end()
})

test('transformCell: single / col / delete after', (t) => {
  let cell = _createCell('C1')
  let dim = 'col'
  let pos = 4
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C1', MSG)
  t.end()
})

test('transformCell: single / col / delete same', (t) => {
  let cell = _createCell('C1')
  let dim = 'col'
  let pos = 2
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, BROKEN_REF, MSG)
  t.end()
})

test('transformCell: single / col / delete surrounding', (t) => {
  let cell = _createCell('C1')
  let dim = 'col'
  let pos = 1
  let count = -4
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, BROKEN_REF, MSG)
  t.end()
})

// cell reference - insert/delete rows

test('transformCell: single / row / insert before', (t) => {
  let cell = _createCell('C3')
  let dim = 'row'
  let pos = 1
  let count = 3
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C6', MSG)
  t.end()
})

test('transformCell: single / row / insert after', (t) => {
  let cell = _createCell('C3')
  let dim = 'row'
  let pos = 4
  let count = 1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3', MSG)
  t.end()
})

test('transformCell: single / row / delete before', (t) => {
  let cell = _createCell('C3')
  let dim = 'row'
  let pos = 0
  let count = -2
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C1', MSG)
  t.end()
})

test('transformCell: single / row / delete after', (t) => {
  let cell = _createCell('C3')
  let dim = 'row'
  let pos = 4
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3', MSG)
  t.end()
})

test('transformCell: single / row / delete same', (t) => {
  let cell = _createCell('C3')
  let dim = 'row'
  let pos = 2
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, BROKEN_REF, MSG)
  t.end()
})
test('transformCell: single / row / delete surrounding', (t) => {
  let cell = _createCell('C3')
  let dim = 'row'
  let pos = 1
  let count = -4
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, BROKEN_REF, MSG)
  t.end()
})

// range - insert/delete cols

test('transformCell: range / col / insert before', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 2
  let count = 2
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'E3:H10', MSG)
  t.end()
})

test('transformCell: range / col / insert inside', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 4
  let count = 3
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:I10', MSG)
  t.end()
})

test('transformCell: range / col / insert after', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 7
  let count = 1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:F10', MSG)
  t.end()
})

test('transformCell: range / col / delete before', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 1
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'B3:E10', MSG)
  t.end()
})

test('transformCell: range / col / delete overlapping start', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 1
  let count = -4
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'B3:B10', MSG)
  t.end()
})

test('transformCell: range / col / delete inside', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 3
  let count = -2
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:D10', MSG)
  t.end()
})

test('transformCell: range / col / delete overlapping end', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 4
  let count = -3
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:D10', MSG)
  t.end()
})

test('transformCell: range / col / delete after', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'col'
  let pos = 7
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:F10', MSG)
  t.end()
})

// range - insert/delete rows

test('transformCell: range / row / insert before', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 1
  let count = 3
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C6:F13', MSG)
  t.end()
})

test('transformCell: range / row / insert inside', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 6
  let count = 2
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:F12', MSG)
  t.end()
})

test('transformCell: range / row / insert after', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 12
  let count = 1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:F10', MSG)
  t.end()
})

test('transformCell: range / row / delete before', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 1
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C2:F9', MSG)
  t.end()
})

test('transformCell: range / row / delete overlapping start', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 1
  let count = -4
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C2:F6', MSG)
  t.end()
})

test('transformCell: range / row / delete inside', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 5
  let count = -2
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:F8', MSG)
  t.end()
})

test('transformCell: range / row / delete overlapping end', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 7
  let count = -6
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:F7', MSG)
  t.end()
})

test('transformCell: range / row / delete after', (t) => {
  let cell = _createCell('C3:F10')
  let dim = 'row'
  let pos = 12
  let count = -1
  _transformCell(cell, dim, pos, count)
  t.equal(cell.source, 'C3:F10', MSG)
  t.end()
})

function _createCell(source) {
  let cell = new Cell(null, {source})
  // Note: faking code analysis usually done by Engine
  forEach(cell._source.symbolMapping, s => {
    cell.inputs.add(new CellSymbol(s, 'doc', cell))
  })
  return cell
}

function _transformCell(cell, dim, pos, count) {
  recordTransformations({ deps: cell.inputs }, dim === 'row' ? 0 : 1, pos, count)
  applyCellTransformations(cell)
}