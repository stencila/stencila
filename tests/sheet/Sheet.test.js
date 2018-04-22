import test from 'tape'
import { insertRows, deleteRows } from '../../src/sheet/sheetManipulations'
import createRawArchive from '../util/createRawArchive'
import loadRawArchive from '../util/loadRawArchive'
import StubEngine from '../util/StubEngine'

/*
  This test is using an archive with a sheet attached to the Engine
  making sure all Sheet interactions and manipulations are working.

  All tests are done on a model level, i.e. the UI is not tested.

  # TODO

  Model manipulations
  - inserting rows/cols (effect on model only)
  - deleting rows/cols (effect on model only)
  - changing a value
  - clearing a range
  Engine adapter
  - inserting rows/cols (effect on engine)
  - deleting rows/cols (effect on model only)

*/
test('Sheet (model): inserting rows should increase row count', (t) => {
  let { sheetSession, sheet } = _setupModel(simple())
  insertRows(sheetSession, 1, 2)
  t.deepEqual(sheet.getRowCount(), 6, 'row count should have been updated')
  t.end()
})

test('Sheet (model): deleting rows should decrease row count', (t) => {
  let { sheetSession, sheet } = _setupModel(simple())
  deleteRows(sheetSession, 1, 2)
  t.deepEqual(sheet.getRowCount(), 2, 'row count should have been updated')
  t.end()
})

function simple() {
  return {
    id: 'sheet',
    path: 'sheet.xml',
    type: 'sheet',
    name: 'My Sheet',
    columns: [{ name: 'x' }, { name: 'y' }, { name: 'z' }],
    cells: [
      ['1', '2', '3'],
      ['4', '5', '6'],
      ['7', '8', '9'],
      ['10', '11', '12']
    ]
  }
}

function _setupModel(sheetData) {
  let context = {
    engine: new StubEngine()
  }
  let rawArchive = createRawArchive([ sheetData ])
  let archive = loadRawArchive(rawArchive, context)
  let sheetSession = archive.getEditorSession('sheet')
  let sheet = sheetSession.getDocument()
  return { archive, sheetSession, sheet }
}
