import test from 'tape'

import CellGraph from '../../src/engine/CellGraph'
import Cell from '../../src/engine/Cell'
import { READY, WAITING, OK, toString } from '../../src/engine/CellStates'

/*

TODO: add tests
- detect cycle
- detect duplicate export
- parallel evaluation
- change inputs
- change output symbol
- resolving an issue by providing the link between two cells

TODO: beyond having correct states I want to ensure that the minimum amount
of updates are communicated. I.e. a cell should not be affected if not necessary.

*/


/*
 add a single cell with no deps
 should be READY after first update
*/
test('CellGraph: single cell with no deps', t => {
  let g = new CellGraph()
  let cell = new Cell({ id: 'cell1' })
  g.addCell(cell)
  g.update()
  t.equal(cell.state, READY, 'cell should be ready')
  t.end()
})


/*
  add two cells, first providing 'x', second consuming 'x',
  first should be ready right away,
  second should become ready after setting value for 'x'
*/
test('CellGraph: two linked cells', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell2', inputs: ['x'] })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [READY, WAITING])

  g.setResult('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [OK, READY])

  g.setResult('cell2', 2)
  updates = g.update()
  _checkUpdates(t, updates, ['cell2'])
  _checkStates(t, cells, [OK, OK])

  t.end()
})

test('CellGraph: Y shaped graph', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell2', output: 'y' }),
    new Cell({ id: 'cell3', inputs: ['x', 'y'], output: 'z' }),
    new Cell({ id: 'cell4', inputs: ['z'] }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3', 'cell4'])
  _checkStates(t, cells, [READY, READY, WAITING, WAITING])

  g.setResult('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [OK, READY, WAITING, WAITING])

  g.setResult('cell2', 2)
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [OK, OK, READY, WAITING])

  g.setResult('cell3', 3)
  updates = g.update()
  _checkUpdates(t, updates, ['cell3', 'cell4'])
  _checkStates(t, cells, [OK, OK, OK, READY])

  g.setResult('cell4', 6)
  updates = g.update()
  _checkUpdates(t, updates, ['cell4'])
  _checkStates(t, cells, [OK, OK, OK, OK])

  t.end()
})

function _checkStates(t, cells, states) {
  for (let i = 0; i < cells.length; i++) {
    const cell = cells[i]
    const state = states[i]
    t.equal(cell.state, state, `${cell.id} should be ${toString(state)}`)
  }
}

function _checkUpdates(t, updates, expected) {
  let actual = Array.from(updates)
  actual.sort()
  t.deepEqual(actual, expected, 'Updates should be correct')
}