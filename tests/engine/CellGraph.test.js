import test from 'tape'

import CellGraph from '../../src/engine/CellGraph'
import Cell from '../../src/engine/Cell'
import { SyntaxError, RuntimeError, ContextError } from '../../src/engine/CellErrors'
import { UNKNOWN, ANALYSED, BROKEN, BLOCKED, WAITING, READY, OK, FAILED, toString } from '../../src/engine/CellStates'

/*
 add a single cell with no deps
 should be READY after first update
*/
test('CellGraph: single cell with no deps', t => {
  let g = new CellGraph()
  let cell = new Cell(null, { id: 'cell1', status: ANALYSED })
  g.addCell(cell)

  g.update()
  t.equal(cell.status, READY, 'cell should be ready')

  g.setValue('cell1', 1)
  g.update()
  t.equal(cell.status, OK, 'cell should be OK after value is set')

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
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED}),
    new Cell(null, { id: 'cell2', inputs: ['x'], status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [READY, WAITING])

  g.setValue('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [OK, READY])

  g.setValue('cell2', 2)
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
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['x', 'y'], output: 'z', status: ANALYSED }),
    new Cell(null, { id: 'cell4', inputs: ['z'], status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3', 'cell4'])
  _checkStates(t, cells, [READY, READY, WAITING, WAITING])

  g.setValue('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [OK, READY, WAITING, WAITING])

  g.setValue('cell2', 2)
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [OK, OK, READY, WAITING])

  g.setValue('cell3', 3)
  updates = g.update()
  _checkUpdates(t, updates, ['cell3', 'cell4'])
  _checkStates(t, cells, [OK, OK, OK, READY])

  g.setValue('cell4', 6)
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
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', inputs: ['x'], output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['x'], output: 'z', status: ANALYSED }),
    new Cell(null, { id: 'cell4', inputs: ['y', 'z'], status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3', 'cell4'])
  _checkStates(t, cells, [READY, WAITING, WAITING, WAITING])

  g.setValue('cell1', 2)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3'])
  _checkStates(t, cells, [OK, READY, READY, WAITING])

  g.setValue('cell2', 4)
  updates = g.update()
  _checkUpdates(t, updates, ['cell2'])
  _checkStates(t, cells, [OK, OK, READY, WAITING])

  g.setValue('cell3', 6)
  updates = g.update()
  _checkUpdates(t, updates, ['cell3', 'cell4'])
  _checkStates(t, cells, [OK, OK, OK, READY])

  g.setValue('cell4', 10)
  updates = g.update()
  _checkUpdates(t, updates, ['cell4'])
  _checkStates(t, cells, [OK, OK, OK, OK])

  t.end()
})

test('CellGraph: missing link', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['y'], status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell3'])
  _checkStates(t, cells, [READY, BROKEN])

  let missingLink = new Cell(null, { id: 'cell2', inputs: ['x'], output: 'y', status: ANALYSED })
  cells.splice(1, 0, missingLink)
  g.addCell(missingLink)
  g.setValue('cell1', 1)
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
    new Cell(null, { id: 'cell1', inputs: ['y'], output: 'x', status: ANALYSED }),
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
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', inputs: ['x'], output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['y'], status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, WAITING, WAITING])

  // this should break cell2, and block cell3
  g.addError('cell2', new SyntaxError('ERROR'))
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [READY, BROKEN, BLOCKED])

  g.setValue('cell1', 1)
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
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', inputs: ['x'], output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['y'], status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, WAITING, WAITING])

  g.addError('cell1', new RuntimeError('ERROR'))
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3'])
  _checkStates(t, cells, [FAILED, BLOCKED, BLOCKED])

  t.end()
})

/*
  Inputs of a cell can be changed, and the graph
  propagates status updates accordingly.
*/
test('CellGraph: changing inputs', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['z'], status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, READY, BROKEN])

  g.setValue('cell1', 1)
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
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', inputs: ['y'], status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['z'], status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, BROKEN, BROKEN])

  g.setValue('cell1', 1)
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
  An example where a cyclic dependency is created and resolved.

  'cell1' is outside the cycle, to see the difference in updated states.
  'cell2' has an unresolved dependency at the beginning.
  Then 'cell4' is added, consuming the output of 'cell3',
  and providing the unresolved input of 'cell2', which forms a cycle.
  All involved cells are set to broken.
  Removing the dependency from 'cell2' brings all cells back
  into an unbroken status.
*/
test('CellGraph: cycle', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x1', status: ANALYSED }),
    new Cell(null, { id: 'cell2', inputs: ['x1', 'x4'], output: 'x2', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['x2'], output: 'x3', status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [READY, BROKEN, BLOCKED])

  let cell4 = new Cell(null, { id: 'cell4', inputs: ['x3'], output: 'x4' })
  cells.push(cell4)
  g.addCell(cell4)
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3', 'cell4'])
  _checkStates(t, cells, [READY, BROKEN, BROKEN, BROKEN])

  g.setInputs('cell2', ['x1'])
  updates = g.update()
  _checkUpdates(t, updates, ['cell2', 'cell3', 'cell4'])
  _checkStates(t, cells, [READY, WAITING, WAITING, WAITING])

  t.end()
})

test('CellGraph: resolving a cycle', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', inputs: ['y'], output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', inputs: ['x'], output: 'y', status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [BROKEN, BROKEN])

  g.setInputs('cell2', [])
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [WAITING, READY])

  t.end()
})

/*
  Two cells exposing the same variable is considered a conflict.
  All involved cells are marked as broken.
  Graph returns to proper status when the problem is resolved.
*/
test('CellGraph: name collision', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell3', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell4', inputs: ['x'], status: ANALYSED }),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkStates(t, cells, [BROKEN, BROKEN, BROKEN, BLOCKED])

  g.setOutput('cell2', 'y')
  updates = g.update()
  _checkUpdates(t, updates, ['cell2'])
  _checkStates(t, cells, [BROKEN, READY, BROKEN, BLOCKED])

  g.setOutput('cell3', 'z')
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell3', 'cell4'])
  _checkStates(t, cells, [READY, READY, READY, WAITING])

  t.end()
})

/*
  Cells with side effects are primarily updated in the order of their occurrence.
*/
test('CellGraph: cells with side effects', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', hasSideEffects: true, next: 'cell2', status: ANALYSED }),
    new Cell(null, { id: 'cell2', hasSideEffects: true, prev: 'cell1', status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [READY, WAITING])

  g.setValue('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [OK, READY])

  t.end()
})

/*
  Cells with side effects can be mixed with pure ones.
  While for cells without side-effects a minimal update can be done,
  for cells with side-effects all subsequent cells (with side-effects) must be invalidated.
  Example:
  ```
  x = 2
  y = 3
  foo(x,y)
  z = 4
  bar(x,z)
  3*x
  ```
  Consider every line as a single cell, and 'foo()' and 'bar()' having side-effects.
  Because 'bar()' is called after 'foo()', and 'foo()' might have an unknown side-effect that might affect 'bar()',
  we will re-evaluate 'bar()' whenever 'foo(x,y)' has changed.

  TODO: discuss if this is really what we want. At least we should make sure, that these cells are always run in the correct order
  -> which is an indicator for automatically invalidating the 'bar()'
*/
test('CellGraph: mixing cells with/without side-effects', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED }),
    new Cell(null, { id: 'cell2', output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['x', 'y'], hasSideEffects: true, next: 'cell5', status: ANALYSED }),
    new Cell(null, { id: 'cell4', output: 'z', status: ANALYSED }),
    new Cell(null, { id: 'cell5', inputs: ['x', 'z'], hasSideEffects: true, prev: 'cell3', status: ANALYSED }),
    new Cell(null, { id: 'cell6', inputs: ['x'], status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3', 'cell4', 'cell5', 'cell6'])
  _checkStates(t, cells, [READY, READY, WAITING, READY, WAITING, WAITING])

  g.setValue('cell1', 1)
  g.setValue('cell2', 2)
  g.setValue('cell4', 4)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3', 'cell4', 'cell6'])
  // it is important here, that cell5 is not READY, because it must wait for 'cell3' first
  _checkStates(t, cells, [OK, OK, READY, OK, WAITING, READY])

  // 'evaluating' cell6 so that we can see that subsequent updates do not affect this cell
  g.setValue('cell6', 6)
  updates = g.update()
  _checkUpdates(t, updates, ['cell6'])
  _checkStates(t, cells, [OK, OK, READY, OK, WAITING, OK])

  g.setValue('cell3', 3)
  updates = g.update()
  _checkUpdates(t, updates, ['cell3', 'cell5'])
  _checkStates(t, cells, [OK, OK, OK, OK, READY, OK])

  g.setValue('cell5', 5)
  updates = g.update()
  _checkUpdates(t, updates, ['cell5'])
  _checkStates(t, cells, [OK, OK, OK, OK, OK, OK])

  t.end()
})

test('CellGraph: removing a cell from a notebook', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', hasSideEffects: true, next: 'cell2', status: ANALYSED }),
    new Cell(null, { id: 'cell2', hasSideEffects: true, next: 'cell3', prev: 'cell1', status: ANALYSED }),
    new Cell(null, { id: 'cell3', hasSideEffects: true, prev: 'cell2', status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2', 'cell3'])
  _checkStates(t, cells, [READY, WAITING, WAITING])

  g.setValue('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [OK, READY, WAITING])

  g.removeCell('cell2')
  updates = g.update()
  cells = [cells[0], cells[2]]
  _checkUpdates(t, updates, ['cell3'])
  _checkStates(t, cells, [OK, READY])

  t.end()
})

test('CellGraph: adding an engine error should imply BROKEN state', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1'}),
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [UNKNOWN])

  g.addError('cell1', new ContextError('Unknown context.'))
  updates = g.update()
  _checkUpdates(t, updates, ['cell1'])
  _checkStates(t, cells, [BROKEN])

  t.end()
})

test('CellGraph: remove a cell', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED}),
    new Cell(null, { id: 'cell2', inputs: ['x'], output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['y'], status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()

  g.removeCell('cell1')
  updates = g.update()
  t.notOk(g.hasCell('cell1'), 'cell1 should have been removed')
  cells = cells.slice(1)
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [BROKEN, BLOCKED])

  g.removeCell('cell2')
  updates = g.update()
  t.notOk(g.hasCell('cell2'), 'cell2 should have been removed')
  cells = cells.slice(1)
  _checkUpdates(t, updates, ['cell3'])
  _checkStates(t, cells, [BROKEN])

  t.end()
})

test('CellGraph: cells with side-effects', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x', status: ANALYSED}),
    new Cell(null, { id: 'cell2', inputs: ['x'], output: 'y', status: ANALYSED }),
    new Cell(null, { id: 'cell3', inputs: ['y'], status: ANALYSED })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()

  g.removeCell('cell1')
  updates = g.update()
  t.notOk(g.hasCell('cell1'), 'cell1 should have been removed')
  cells = cells.slice(1)
  _checkUpdates(t, updates, ['cell2', 'cell3'])
  _checkStates(t, cells, [BROKEN, BLOCKED])

  g.removeCell('cell2')
  updates = g.update()
  t.notOk(g.hasCell('cell2'), 'cell2 should have been removed')
  cells = cells.slice(1)
  _checkUpdates(t, updates, ['cell3'])
  _checkStates(t, cells, [BROKEN])

  t.end()
})

/*
test('CellGraph: TEMPLATE', t => {
  let g = new CellGraph()
  let cells = [
    new Cell(null, { id: 'cell1', output: 'x' }),
    new Cell(null, { id: 'cell2', inputs: ['x'] })
  ]
  cells.forEach(c => g.addCell(c))

  let updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [READY, WAITING])

  g.setValue('cell1', 1)
  updates = g.update()
  _checkUpdates(t, updates, ['cell1', 'cell2'])
  _checkStates(t, cells, [OK, READY])

  t.end()
})
*/

function _checkStates(t, cells, expected) {
  expected = expected.map(toString)
  let actual = cells.map(cell => toString(cell.status))
  t.deepEqual(actual, expected, 'cell states should be correct')
}

function _checkUpdates(t, updates, expected) {
  let actual = Array.from(updates)
  actual.sort()
  t.deepEqual(actual, expected, `${expected.join(', ')} should have been updated`)
}