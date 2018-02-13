import test from 'tape'

import CellGraph from '../../src/engine/CellGraph'
import Cell from '../../src/engine/Cell'
import { BROKEN, WAITING, READY, OK, toString } from '../../src/engine/CellStates'

/*

TODO: add tests
- blocked cell (variants: by engine, graph, or runtime error)
- change inputs
- change output symbol
- resolving an issue by providing the link between two cells
- detect cycle
- detect duplicate export
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

/*
  3 levels, where  shaped like this `>-`
  Merge node should become ready only after both inputs are ok.
*/
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

/*
  A diamond shaped graph, which could allow parallel computation `<>`
  After the first cell being ok, the next cells should be ready simultanously,
  while the last merging cell has to wait for both being finished.
*/
test('CellGraph: Diamond', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell2', inputs: ['x'], output: 'y' }),
    new Cell({ id: 'cell3', inputs: ['x'], output: 'z' }),
    new Cell({ id: 'cell4', inputs: ['y', 'z'] }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3', 'cell4'])
  _checkStates(t, cells, [READY, WAITING, WAITING, WAITING])

  g.setResult('cell1', 2)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3'])
  _checkStates(t, cells, [OK, READY, READY, WAITING])

  g.setResult('cell2', 4)
  updates = g.update()
  _checkUpdates(t, updates, ['cell2'])
  _checkStates(t, cells, [OK, OK, READY, WAITING])

  g.setResult('cell3', 6)
  updates = g.update()
  _checkUpdates(t, updates, ['cell3', 'cell4'])
  _checkStates(t, cells, [OK, OK, OK, READY])

  g.setResult('cell4', 10)
  updates = g.update()
  _checkUpdates(t, updates, ['cell4'])
  _checkStates(t, cells, [OK, OK, OK, OK])

  t.end()
})

test('CellGraph: unresolvable input', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', inputs: ['y'], output: 'x' }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [BROKEN])

  t.end()
})


/*
test('CellGraph: TEMPLATE', t => {
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

  t.end()
})
*/

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