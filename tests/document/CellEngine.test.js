import test from 'tape'

import { isNil, map } from 'substance'
import { wait, setupEditorSession } from '../testHelpers'
import CellEngine from '../../src/document/CellEngine'

const WAIT_FOR_IDLE = 1

test('CellEngine: setup engine without cells', (t) => {
  t.plan(2)
  const {cellEngine} =setupCellEngine()
  t.notOk(isNil(cellEngine), 'Should have setup a CellEngine')
  t.equal(_getCells(cellEngine).length, 0, 'There should be no cells initially')
})

test('CellEngine: setup engine but with initial cells', (t) => {
  t.plan(1)
  const {editorSession} = setupEditorSession('/tests/documents/simple/default.html')
  const doc = editorSession.getDocument()
  doc.create({
    type: 'cell',
  })
  doc.create({
    type: 'inline-cell',
    path: ['p1', 'content'],
    startOffset: 1,
    endOffset: 2
  })
  doc.create({
    type: 'select',
    name: 'foo',
    path: ['p1', 'content'],
    startOffset: 4,
    endOffset: 5
  })
  doc.create({
    type: 'range-input',
    name: 'bar',
    path: ['p1', 'content'],
    startOffset: 7,
    endOffset: 8
  })
  const cellEngine = new CellEngine(editorSession, {waitForIdle: WAIT_FOR_IDLE})
  t.equal(_getCells(cellEngine).length, 4, 'There should be 4 cells')
})

test('CellEngine: dispose', (t) => {
  t.plan(1)
  const {editorSession} = setupEditorSession('/tests/documents/simple/default.html')
  const doc = editorSession.getDocument()
  doc.create({
    type: 'cell',
  })
  doc.create({
    type: 'select',
    name: 'foo',
    path: ['p1', 'content'],
    startOffset: 4,
    endOffset: 5
  })
  const cellEngine = new CellEngine(editorSession, {waitForIdle:WAIT_FOR_IDLE})
  cellEngine.dispose()
  t.equal(_getCells(cellEngine).length, 0, 'There should be no cells registered')
})

test('CellEngine: detect creation and deletion of cells', (t) => {
  _shouldUpdatedOnCreateDelete(t, 'cell')
})

test('CellEngine: detect creation and deletion of inline-cells', (t) => {
  _shouldUpdatedOnCreateDelete(t, 'inline-cell', {
    path: ['p1', 'content'],
    startOffset: 1,
    endOffset: 2
  })
})

test('CellEngine: detect creation and deletion of select', (t) => {
  _shouldUpdatedOnCreateDelete(t, 'select', {
    name: 'foo',
    path: ['p1', 'content'],
    startOffset: 1,
    endOffset: 2
  })
})

test('CellEngine: detect creation and deletion of range-input', (t) => {
  _shouldUpdatedOnCreateDelete(t, 'range-input', {
    name: 'foo',
    path: ['p1', 'content'],
    startOffset: 1,
    endOffset: 2
  })
})

test('CellEngine: call function', (t) => {
  t.plan(1)
  const {editorSession, doc} = setupCellEngine()
  editorSession.transaction((tx) => {
    tx.create({
      id: 'cell1',
      type: 'cell',
      expression: 'type(3.14)'
    })
  })
  const cell1 = doc.get('cell1')
  Promise.resolve()
  .then(wait(10))
  .then(() => {
    t.equal(cell1.value, 'float', 'cell should have been evaluated.')
  })
})

test('CellEngine: call external cell', (t) => {
  t.plan(1)
  const {cellEngine, editorSession, doc} = setupCellEngine()
  cellEngine.setValue('x', 2)
  editorSession.transaction((tx) => {
    tx.create({
      id: 'cell1',
      type: 'cell',
      expression: 'js(x)',
      language: 'js',
      sourceCode: 'return x*33'
    })
  })
  const cell1 = doc.get('cell1')
  Promise.resolve()
  .then(wait(100))
  .then(() => {
    t.equal(cell1.value, 66, 'cell should have been evaluated.')
  })
})

test('CellEngine: call external code', (t) => {
  t.plan(1)
  const {editorSession, doc} = setupCellEngine()
  editorSession.transaction((tx) => {
    tx.create({
      id: 'cell1',
      type: 'cell',
      expression: 'global js()',
      language: 'js',
      sourceCode: '99'
    })
  })
  const cell1 = doc.get('cell1')
  Promise.resolve()
  .then(wait(10))
  .then(() => {
    t.equal(cell1.value, 99, 'cell should have been evaluated.')
  })
})

test('CellEngine: update a cell', (t) => {
  t.plan(2)
  const {editorSession, doc} = setupCellEngine()
  editorSession.transaction((tx) => {
    tx.create({
      id: 'cell1',
      type: 'cell',
      expression: 'x = 42'
    })
  })
  const cell1 = doc.get('cell1')
  Promise.resolve()
  .then(wait(10))
  .then(() => {
    t.equal(cell1.value, 42, 'cell should have been evaluated.')
    editorSession.transaction((tx) => {
      tx.set(['cell1', 'expression'], 'x = 77')
    })
  })
  .then(wait(10))
  .then(() => {
    t.equal(cell1.value, 77, 'cell should have been updated.')
  })
})

test('CellEngine: update an input', (t) => {
  t.plan(3)
  const {editorSession, doc} = setupCellEngine()
  editorSession.transaction((tx) => {
    tx.create({
      id: 'input1',
      type: 'range-input',
      name: 'x',
      value: 42,
      path: ['p1', 'content'],
      startOffset: 1,
      endOffset: 2
    })
    tx.create({
      id: 'cell1',
      type: 'cell',
      expression: 'x+1'
    })
  })
  const cell1 = doc.get('cell1')
  Promise.resolve()
  .then(wait(10))
  .then(() => {
    t.equal(cell1.value, 43, 'cell should have been evaluated.')
    editorSession.transaction((tx) => {
      tx.set(['input1', 'value'], 77)
    })
  })
  .then(wait(10))
  .then(() => {
    t.equal(cell1.value, 78, 'cell should have been updated.')
    editorSession.transaction((tx) => {
      tx.set(['input1', 'name'], 'y')
    })
  })
  .then(wait(10))
  .then(() => {
    t.notOk(Boolean(cell1.value), 'cell should have been invalidated.')
  })
})

function setupCellEngine() {
  const {editorSession, doc} = setupEditorSession('/tests/documents/simple/default.html')
  const cellEngine =new CellEngine(editorSession, {waitForIdle: WAIT_FOR_IDLE})
  return {editorSession, doc, cellEngine}
}

function _shouldUpdatedOnCreateDelete(t, type, nodeData = {}) {
  t.plan(2)
  nodeData.type = type
  const {cellEngine, editorSession} = setupCellEngine()
  editorSession.transaction((tx) => {
    tx.create(nodeData)
  })
  let cells = _getCells(cellEngine)
  t.equal(cells.length, 1, 'There should be one cell registered after creation of cell.')
  editorSession.transaction((tx) => {
    tx.delete(cells[0].id)
  })
  cells = _getCells(cellEngine)
  t.equal(cells.length, 0, 'There should be no cells registered after deletion of cell.')
}

function _getCells(cellEngine) {
  return map(cellEngine._cells).concat(map(cellEngine._inputs))
}
