import test from 'tape'

import CellGraph from '../../src/engine/CellGraph'
import Cell from '../../src/engine/Cell'
import { READY, WAITING, OK } from '../../src/engine/CellStates'

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
  let cell1 = new Cell({ id: 'cell1', output: 'x' })
  let cell2 = new Cell({ id: 'cell2', inputs: ['x'] })
  g.addCell(cell1)
  g.addCell(cell2)
  g.update()
  t.equal(cell1.state, READY, 'cell1 should be ready')
  t.equal(cell2.state, WAITING, 'cell2 should be waiting')
  g.setResult('cell1', 1)
  g.update()
  t.equal(cell1.state, OK, 'cell1 should be ok')
  t.equal(cell2.state, READY, 'cell2 should be ready')
  t.end()
})
