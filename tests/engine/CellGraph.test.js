import test from 'tape'

import CellGraph from '../../src/engine/CellGraph'
import Cell from '../../src/engine/Cell'
import { SyntaxError, RuntimeError } from '../../src/engine/CellErrors'
import { BROKEN, BLOCKED, WAITING, READY, OK, FAILED, toString } from '../../src/engine/CellStates'

/*

TODO: add tests
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

test('CellGraph: missing link', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell3', inputs: ['y'] })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell3'])
  _checkStates(t, cells, [READY, BROKEN])

  let missingLink = new Cell({ id: 'cell2', inputs: ['x'], output: 'y' })
  cells.splice(1, 0, missingLink)
  g.addCell(missingLink)
  g.setResult('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3'])
  _checkStates(t, cells, [OK, READY, WAITING])

  t.end()
})

/*
  The cell graph detects if a cell is depending on a variable that
  can not be resolved, and marks the cell as 'broken'
*/
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
  A broken cell blocks subsequent cells.
  While graph errors are managed by the graph automatically,
  other errors, such as SyntaxErrors must be managed, e.g., cleared,
  by the engine.
  When an error has been resolved, the graph un-blocks depending
  cells automatically.
*/
test('CellGraph: blocked cell', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell2', inputs: ['x'], output: 'y' }),
    new Cell({ id: 'cell3', inputs: ['y'] }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, WAITING, WAITING])

  // this should break cell2, and block cell3
  g.addError('cell2', new SyntaxError('ERROR'))
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [READY, BROKEN, BLOCKED])

  g.setResult('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [OK, BROKEN, BLOCKED])

  g.clearErrors('cell2', 'engine')
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [OK, READY, WAITING])

  t.end()
})

/*
  If a an evaluation of a cell fails with a runtime error,
  all subsequent cells are blocked.
*/
test('CellGraph: failed evaluation', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell2', inputs: ['x'], output: 'y' }),
    new Cell({ id: 'cell3', inputs: ['y'] }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, WAITING, WAITING])

  g.setResult('cell1', undefined, [new RuntimeError('ERROR')])
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3'])
  _checkStates(t, cells, [FAILED, BLOCKED, BLOCKED])

  t.end()
})

/*
  Inputs of a cell can be changed, and the graph
  propagates state updates accordingly.
*/
test('CellGraph: changing inputs', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell2', output: 'y' }),
    new Cell({ id: 'cell3', inputs: ['z'] }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, READY, BROKEN])

  g.setResult('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [OK, READY, BROKEN])

  g.setInputs('cell3', ['x'])
  updates = g.update()
  _checkUpdates(t, updates, ['cell3'])
  _checkStates(t, cells, [OK, READY, READY])

  g.setInputs('cell3', ['x', 'y'])
  updates = g.update()
  _checkUpdates(t, updates, ['cell3'])
  _checkStates(t, cells, [OK, READY, WAITING])

  t.end()
})

/*
  Output of a cell can be changed, and the graph reacts automatically.
*/
test('CellGraph: changing output', t => {
  let g = new CellGraph()
  let cells = [
    new Cell({ id: 'cell1', output: 'x' }),
    new Cell({ id: 'cell2', inputs: ['y'] }),
    new Cell({ id: 'cell3', inputs: ['z'] })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, BROKEN, BROKEN])

  g.setResult('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [OK, BROKEN, BROKEN])

  g.setOutput('cell1', 'y')
  updates = g.update()
  _checkUpdates(t, updates, ['cell2'])
  _checkStates(t, cells, [OK, READY, BROKEN])

  g.setOutput('cell1', 'z')
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [OK, BROKEN, READY])

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
  t.deepEqual(actual, expected, `${expected.join(', ')} should have been updated`)
}