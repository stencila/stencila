import test from 'tape'
import { UNKNOWN } from '../../src/engine/CellStates'
import { RuntimeError } from '../../src/engine/CellErrors'
import { BROKEN_REF } from '../../src/engine/engineHelpers'
import _setup from '../util/setupEngine'
import { getValue, getValues, getSources, getStates, getErrors, cycle, play } from '../util/engineTestHelpers'
import { queryCells } from '../util/sheetTestHelpers'

test('Engine: simple sheet', t=> {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    // default lang
    lang: 'mini',
    cells: [
      ['1', '= A1 * 2'],
      ['2', '= A2 * 2']
    ]
  })
  play(engine)
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'B1:B2')), [2,4], 'values should have been computed')
  })
})

test('Engine: simple doc', t => {
  t.plan(1)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      'x = 2',
      'y = 3',
      'z = x + y'
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getValues(cells), [2,3,5], 'values should have been computed')
  })
})

test('Engine: single cell', t => {
  t.plan(9)
  let { engine, graph } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      '1+2'
    ]
  })
  let cells = doc.getCells()
  const cell = cells[0]
  const id = cell.id
  cycle(engine)
  .then(() => {
    let nextActions = engine.getNextActions()
    t.equal(nextActions.size, 1, 'There should be one next action')
    let a = nextActions.get(id)
    t.equal(a.type, 'register', '.. which should a registration action')
    t.equal(cell.status, UNKNOWN, 'cell state should be UNKNOWN')
  })
  .then(() => cycle(engine))
  .then(() => {
    t.ok(graph.hasCell(id), 'The cell should now be registered')
    let nextActions = engine.getNextActions()
    let a = nextActions.get(id)
    t.equal(a.type, 'evaluate', 'next action should be evaluate')
  })
  .then(() => cycle(engine))
  .then(() => {
    let nextActions = engine.getNextActions()
    let a = nextActions.get(id)
    t.equal(a.type, 'update', 'next action should be update')
  })
  .then(() => cycle(engine))
  .then(() => {
    let nextActions = engine.getNextActions()
    t.equal(nextActions.size, 0, 'There should be no pending actions')
    t.notOk(cell.hasErrors(), 'the cell should have no error')
    t.equal(getValue(cell), 3, 'the value should have been computed correctly')
  })
})

test('Engine: sheet', t=> {
  t.plan(4)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    // default lang
    lang: 'mini',
    cells: [
      ['1', '= A1 * 2'],
      ['2', '= A2 * 2']
    ]
  })
  let [ [, cell2], [, cell4] ] = sheet.getCells()
  cycle(engine)
  .then(() => {
    _checkActions(t, engine, [cell2, cell4], ['register', 'register'])
  })
  .then(() => {
    return cycle(engine)
  })
  .then(() => {
    _checkActions(t, engine, [cell2, cell4], ['evaluate', 'evaluate'])
  })
  .then(() => {
    return cycle(engine)
  })
  .then(() => {
    _checkActions(t, engine, [cell2, cell4], ['update', 'update'])
  })
  .then(() => {
    return cycle(engine)
  })
  .then(() => {
    t.deepEqual(getValues([cell2, cell4]), [2,4], 'values should have been computed')
  })
})

test('Engine: range expression', t=> {
  t.plan(4)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2', '= A1:B1'],
      ['3', '4', '= B2:B2'],
      ['= A1:A2', '6', '= A1:B2'],
    ]
  })
  let [ [,,cell1], [,,cell2], [cell3,,cell4] ] = sheet.getCells()
  let cells = [cell1, cell2, cell3, cell4]
  cycle(engine)
  .then(() => {
    _checkActions(t, engine, cells, ['register', 'register','register', 'register'])
  })
  .then(() => cycle(engine))
  .then(() => {
    _checkActions(t, engine, cells, ['evaluate', 'evaluate','evaluate', 'evaluate'])
  })
  .then(() => cycle(engine))
  .then(() => {
    _checkActions(t, engine, cells, ['update', 'update', 'update','update'])
  })
  .then(() => cycle(engine))
  .then(() => {
    t.deepEqual(
      getValues(cells),
      [[1,2], 4, [1,3], {"type":"table","data":{"A":[1,3],"B":[2,4]},"columns":2,"rows":2}],
      'values should have been computed'
    )
  })
})

/*
  Scenario:
  1. create a doc with two cells 'x = 1' and 'x = 2'
    -> now there should be an error because of the name collision
  2. update both cells (not resolving the issue)
    -> both should still have the same error
*/
test('Engine: graph errors should not be cleared without resolving', t => {
  t.plan(2)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      { id: 'cell1', source: 'x = 1' },
      { id: 'cell2', source: 'x = 2' }
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getErrors(cells), [['collision'], ['collision']], 'Both cells should have a collision error.')
  })
  .then(() => {
    doc.updateCell('cell1', { source: 'x =  1'})
    doc.updateCell('cell2', { source: 'x = 3'})
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getErrors(cells), [['collision'], ['collision']], 'still both cells should have a collision error.')
  })
})

test('Engine: runtime errors should be wiped when inputs are updated', t => {
  t.plan(2)
  let { engine, graph } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      { id: 'cell1', source: 'x = 1' },
      { id: 'cell2', source: 'y = x' }
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    t.equal(getValue(cells[1]), 1, 'y should be computed.')
    graph.addError(cells[1].id, new RuntimeError('Ooops'))
  })
  .then(() => play(engine))
  .then(() => {
    doc.updateCell('cell1', { source: 'x = 2' })
  })
  .then(() => play(engine))
  .then(() => {
    t.equal(getValue(cells[1]), 2, 'y should be updated.')
  })
})

test('Engine (Document): inserting a cell', t => {
  t.plan(1)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      { id: 'cell1', source: 'x = 2' },
      { id: 'cell2', source: 'z = 3*x' }
    ]
  })
  play(engine)
  .then(() => {
    doc.insertCellAt(1, { id: 'cell3', source: 'y = x + 1' })
  })
  .then(() => play(engine))
  .then(() => {
    doc.updateCell('cell1', { source: 'x = 2' })
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(doc.getCells()), [2,3,6], 'values should have been computed')
  })
})

test('Engine (Document): removing a cell', t => {
  t.plan(1)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      { id: 'cell1', source: 'x = 2' },
      { id: 'cell2', source: 'y = 3*x' },
      { id: 'cell3', source: 'z = 2*y' }
    ]
  })
  play(engine)
  .then(() => {
    doc.removeCell('cell2')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getErrors(doc.getCells()), [[],['unresolved']], 'cell3 should be broken now')
  })
})

test('Engine (Document): updating a cell', t => {
  t.plan(1)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      { id: 'cell1', source: 'x = 2' },
    ]
  })
  play(engine)
  .then(() => {
    doc.updateCell('cell1', 'x = 21')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(doc.getCells()), [21], 'cell should have been updated')
  })
})

test('Engine (Sheet): column names', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    columns: [
      { name: 'x' },
      { name: 'y' },
    ],
    cells: [
      [ '1', '2'],
      [ '3', '4']
    ]
  })
  t.equal(sheet.getColumnName(0), 'x', 'first column name should be correct')
  t.equal(sheet.getColumnName(1), 'y', 'second column name should be correct')
})

test('Engine (Sheet): cell expressions', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['= A1 + 1', '= B1 + 1']
    ]
  })
  let cells = sheet.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getValues(cells[1]), [2,3], 'values should have been computed')
  })
  .then(() => {
    // TODO: still the difference between qualified vs unqualified id
    // is sometimes confusing
    // Note: Document and Sheet API uses unqualified ids (local to the resource, like 'A1')
    // while the engine and the graph uses qualified ids (globally unique, like 'sheet1!A1').
    sheet.updateCell(cells[0][0].unqualifiedId, '3')
    sheet.updateCell(cells[0][1].unqualifiedId, '4')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells[1]), [4,5], 'values should have been computed')
  })
})

test('Engine: changing a range expression', t=> {
  // Note: internally we instantiate a proxy cell
  // which should be pruned automatically if it is not needed anymore
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [['1'],['2'],['3'],['= A1:A2']]
  })
  let [,,,[cell4]] = sheet.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getValue(cell4), [1,2], 'range expression should be evaluated')
  })
  .then(() => {
    sheet.updateCell(cell4.unqualifiedId, '= A1:A3')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValue(cell4), [1,2,3], 'range expression should be updated')
  })
})

test('Engine: inverse range expression are normalized', t=> {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['= A2:A1', '= B1:A1']
    ]
  })
  let cells = sheet.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getValues(cells[2]), [[1,3], [1,2]], 'values should be in normal order')
  })
})

test('Engine: no context for lang', t => {
  t.plan(1)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'foo',
    cells: [
      'x = 2'
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getErrors(cells), [['context']], 'there should an error about missing context')
  })
})

test('Engine: lost context', t => {
  t.plan(2)
  let { engine, host } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      'x = 2'
    ]
  })
  let cells = doc.getCells()
  cycle(engine)
  .then(() => cycle(engine))
  .then(() => {
    // now the cell should be scheduled for evaluation
    _checkActions(t, engine, cells, ['evaluate'])
    // and there we pretend a lost connection
    host._disable(true)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getErrors(cells), [['context']], 'there should an error about missing context')
  })
})

test('Engine: transclusion', t => {
  t.plan(2)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      'x = sheet1!A3',
      'x * 2'
    ]
  })
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['= A1 + A2', '= B1 + B2']
    ]
  })
  let docCells = doc.getCells()
  let sheetCells = sheet.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getValues(docCells), [4, 8], 'document cells should have been computed')
  })
  .then(() => {
    sheet.updateCell(sheetCells[0][0].unqualifiedId, '5')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(docCells), [8, 16], 'document cells should have been computed')
  })
})

test('Engine: manual execution', t => {
  t.plan(3)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    autorun: false,
    cells: [
      'x = 2',
      'x * 3'
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getStates(cells), ['ready', 'waiting'], 'cell states should be correct')
  })
  .then(() => {
    engine._allowRunningCell(cells[0].id)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getStates(cells), ['ok', 'ready'], 'cell states should be correct')
  })
  .then(() => {
    engine._allowRunningCell(cells[1].id)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [2, 6], 'cells should have been computed')
  })
})

test('Engine: manually run cell and predecessors', t => {
  t.plan(1)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    autorun: false,
    cells: [
      'x = 2',
      'y = x * 3',
      'z = y + 2'
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    engine._allowRunningCellAndPredecessors(cells[2].id)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [2, 6, 8], 'cells should have been computed')
  })
})

test('Engine: run all cells in manual execution mode', t => {
  t.plan(1)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    autorun: false,
    cells: [
      'x = 2',
      'y = x * 3',
      'z = y + 2'
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    engine._allowRunningAllCellsOfDocument('doc1')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [2, 6, 8], 'cells should have been computed')
  })
})

test('Engine: cells with errors should not be scheduled (manual mode)', t => {
  t.plan(3)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    autorun: false,
    cells: [
      '6 * 2',
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    engine._allowRunningAllCellsOfDocument('doc1')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [12], 'cells should have been computed')
  })
  .then(() => {
    doc.updateCell(cells[0].unqualifiedId, { source: '6 * 2 +'})
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getStates(cells), ['broken'], 'cell should be broken')
  })
  .then(() => {
    doc.updateCell(cells[0].unqualifiedId, { source: '6 * 2 + 1'})
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getStates(cells), ['ready'], 'cell should be ready')
  })
})

test('Engine: insert rows', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4']
    ]
  })
  play(engine)
  .then(() => {
    sheet.insertRows(1, [['5', '6'], ['7', '8']])
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'A2:B3')), [[5, 6],[7, 8]], 'cells should have been inserted')
  })
})

test('Engine: append rows', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4']
    ]
  })
  play(engine)
  .then(() => {
    sheet.insertRows(2, [['5', '6'], ['7', '8'], ['9', '10']])
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'A3:B5')), [[5, 6],[7, 8], [9, 10]], 'cells should have been inserted')
  })
})

test('Engine: delete rows', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8']
    ]
  })
  play(engine)
  .then(() => {
    sheet.deleteRows(0, 2)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(sheet.getCells()), [[5, 6],[7, 8]], 'rows should have been removed')
  })
})

test('Engine: insert cols', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      [{id:'c1',source:'1'}, {id:'c2',source:'2'}],
      [{id:'c3',source:'3'},{id:'c4',source:'4'}]
    ]
  })
  play(engine)
  .then(() => {
    sheet.insertCols(1, [[{id:'c5',source:'5'}], [{id:'c6',source:'6'}]])
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'A1:C2')), [[1, 5, 2],[3, 6, 4]], 'cells should have been inserted')
  })
})

test('Engine: append cols', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4']
    ]
  })
  play(engine)
  .then(() => {
    sheet.insertCols(2, [['5', '6', '7'], ['8', '9', '10']])
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'C1:E2')), [[5, 6, 7],[8, 9, 10]], 'cells should have been inserted')
  })
})

test('Engine: delete cols', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2', '3', '4'],
      ['5', '6', '7', '8'],
      ['9', '10', '11', '12']
    ]
  })
  play(engine)
  .then(() => {
    sheet.deleteCols(1, 2)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(sheet.getCells()), [[1,4],[5,8],[9,12]], 'cols should have been removed')
  })
})

test('Engine: insert a row', t => {
  t.plan(3)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8'],
      ['=sum(A1:A4)', '=sum(B1:B4)'],
    ]
  })
  let cells = sheet.cells[4]
  play(engine)
  .then(() => {
    t.deepEqual(getValues(cells), [16,20], 'cells should have correct values')
  })
  .then(() => {
    sheet.insertRows(1, [['2', '3']])
    t.deepEqual(getSources(cells), ['=sum(A1:A5)','=sum(B1:B5)'], 'sources should have been updated')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [18,23], 'cells should have correct values')
  })
})

test('Engine: insert multiple rows', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8'],
      ['=sum(A1:A4)', '=sum(B1:B4)'],
    ]
  })
  let cells = sheet.cells[4]
  play(engine)
  .then(() => {
    sheet.insertRows(1, [['2', '3'],['4', '5']])
    t.deepEqual(getSources(cells), ['=sum(A1:A6)','=sum(B1:B6)'], 'sources should have been updated')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [22,28], 'cells should have correct values')
  })
})

test('Engine: delete a row', t => {
  t.plan(3)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8'],
      ['=sum(A1:A4)', '=sum(B1:B4)'],
    ]
  })
  let cells = sheet.cells[4]
  play(engine)
  .then(() => {
    t.deepEqual(getValues(cells), [16,20], 'cells should have correct values')
  })
  .then(() => {
    sheet.deleteRows(2, 1)
    t.deepEqual(getSources(cells), ['=sum(A1:A3)','=sum(B1:B3)'], 'sources should have been updated')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [11,14], 'cells should have correct values')
  })
})

test('Engine: delete last row of a cell range', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8'],
      ['=sum(A1:A4)', '=sum(B1:B4)'],
    ]
  })
  let cells = sheet.cells[4]
  play(engine)
  .then(() => {
    sheet.deleteRows(3, 1)
    t.deepEqual(getSources(cells), ['=sum(A1:A3)','=sum(B1:B3)'], 'sources should have been updated')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(cells), [9,12], 'cells should have correct values')
  })
})

test('Engine: delete rows covering an entire cell range', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8'],
      ['=sum(A2:A3)', '=B2+B3'],
    ]
  })
  let cells = sheet.cells[4]
  play(engine)
  .then(() => {
    sheet.deleteRows(1, 2)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getSources(cells), [`=sum(${BROKEN_REF})`,`=${BROKEN_REF}+${BROKEN_REF}`], 'sources should have been updated')
  })
})

test('Engine: insert a column', t => {
  t.plan(3)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8'],
      ['=sum(A1:A4)', '=sum(A1:B4)'],
    ]
  })
  let cells = sheet.cells[4]
  play(engine)
  .then(() => {
    t.deepEqual(getValues(cells), [16,36], 'cells should have correct values')
  })
  .then(() => {
    sheet.insertCols(1, [['2'], ['3'], ['4'], ['5'], ['=sum(B1:B4)']])
  })
  .then(() => play(engine))
  .then(() => {
    let cells = queryCells(sheet.cells, 'A5:C5')
    t.deepEqual(getSources(cells), ['=sum(A1:A4)','=sum(B1:B4)', '=sum(A1:C4)'], 'sources should have been updated')
    t.deepEqual(getValues(cells), [16,14,50], 'cells should have correct values')
  })
})

test('Engine: insert multiple columns', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2'],
      ['3', '4'],
      ['5', '6'],
      ['7', '8'],
      ['=sum(A1:A4)', '=sum(A1:B4)'],
    ]
  })
  play(engine)
  .then(() => {
    sheet.insertCols(1, [['2', '3'],['4', '5'],['6', '7'],['8', '9'],['=sum(B1:B4)','=sum(C1:C4)']])
  })
  .then(() => play(engine))
  .then(() => {
    let cells = queryCells(sheet.cells, 'A5:D5')
    t.deepEqual(getSources(cells), ['=sum(A1:A4)','=sum(B1:B4)', '=sum(C1:C4)', '=sum(A1:D4)'], 'sources should have been updated')
    t.deepEqual(getValues(cells), [16,20,24,80], 'cells should have correct values')
  })
})

test('Engine: delete a column', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2', '3'],
      ['3', '4', '5'],
      ['6', '7', '8'],
      ['9', '10', '11'],
      ['=sum(A1:A4)', '=sum(B1:B4)', '=sum(A1:C4)'],
    ]
  })
  play(engine)
  .then(() => {
    sheet.deleteCols(1, 1)
    t.deepEqual(getSources(queryCells(sheet.cells, 'A5:B5')), ['=sum(A1:A4)','=sum(A1:B4)'], 'sources should have been updated')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'A5:B5')), [19,46], 'cells should have correct values')
  })
})

test('Engine: delete columns covering an entire cell range', t => {
  t.plan(1)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2', '3', '4'],
      ['5', '6', '7', '8'],
      ['=sum(A1:A2)', '=B2+B3', '=C2+C3', '=A3+B3+C3'],
    ]
  })
  play(engine)
  .then(() => {
    sheet.deleteCols(1, 2)
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getSources(queryCells(sheet.cells, 'A3:B3')), [`=sum(A1:A2)`,`=A3+${BROKEN_REF}+${BROKEN_REF}`], 'sources should have been updated')
  })
})

test('Engine: delete last column of a cell range', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['1', '2', '3'],
      ['3', '4', '5'],
      ['6', '7', '8'],
      ['9', '10', '11'],
      ['=sum(A1:A4)', '=sum(B1:B4)', '=sum(A1:B4)'],
    ]
  })
  play(engine)
  .then(() => {
    sheet.deleteCols(1, 1)
    t.deepEqual(getSources(queryCells(sheet.cells, 'A5:B5')), ['=sum(A1:A4)','=sum(A1:A4)'], 'sources should have been updated')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'A5:B5')), [19,19], 'cells should have correct values')
  })
})

test('Engine: resolving a cycle', t => {
  t.plan(2)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      { id: 'cell1', source: 'x = y' },
      { id: 'cell2', source: 'y = x' }
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getErrors(cells), [['cyclic'], ['cyclic']], 'Both cells should have a cyclic dependency error.')
  })
  .then(() => {
    doc.updateCell('cell2', { source: 'y = 1'})
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getErrors(cells), [[], []], 'Cyclic dependency error should be resolved.')
  })
})

test('Engine: resolving a cycle when cell gets invalid', t => {
  t.plan(2)
  let { engine } = _setup()
  let doc = engine.addDocument({
    id: 'doc1',
    lang: 'mini',
    cells: [
      { id: 'cell1', source: 'x = y' },
      { id: 'cell2', source: 'y = x' }
    ]
  })
  let cells = doc.getCells()
  play(engine)
  .then(() => {
    t.deepEqual(getErrors(cells), [['cyclic'], ['cyclic']], 'Both cells should have a cyclic dependency error.')
  })
  .then(() => {
    doc.updateCell('cell2', { source: 'y = '})
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getErrors(cells), [[], ['syntax']], 'Cyclic dependency error should be resolved.')
  })
})

test('Engine: clear old errors when a cell is changed into a constant', t => {
  t.plan(2)
  let { engine } = _setup()
  let sheet = engine.addSheet({
    id: 'sheet1',
    lang: 'mini',
    cells: [
      ['= A2', '2'],
      ['= A1', '4']
    ]
  })
  let cells = queryCells(sheet.cells, 'A1:A2')
  play(engine)
  .then(() => {
    t.deepEqual(getErrors(cells), [['cyclic'], ['cyclic']], 'cells should have a cyclic dependency error')
  })
  .then(() => {
    sheet.updateCell(cells[1].unqualifiedId, '3')
  })
  .then(() => play(engine))
  .then(() => {
    t.deepEqual(getErrors(cells), [[], []], 'errors should have been cleared')
  })
})


function _checkActions(t, engine, cells, expected) {
  let nextActions = engine.getNextActions()
  let actual = []
  for (let i = 0; i < cells.length; i++) {
    const cell = cells[i]
    const action = nextActions.get(cell.id)
    actual.push(action ? action.type : undefined)
  }
  t.deepEqual(actual, expected, 'next actions should be registered correctly')
}
