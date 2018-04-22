import test from 'tape'
import StencilaArchive from '../../src/StencilaArchive'
import { _initStencilaArchive } from '../../src/stencilaAppHelpers'
import { insertRows, deleteRows, insertCols, deleteCols } from '../../src/sheet/sheetManipulations'
import { getSource } from '../../src/shared/cellHelpers'
import createRawArchive from '../util/createRawArchive'

/*
  Transclusions need to be updated whenever the referenced sheet changes
  structurally, i.e. rows or columns are added or removed, or the referenced
  resource is renamed.

  To test this behavior, an archive is created and manipulations as done by the
  UI are triggered. Only the following is subject to this this:

  - transclusions are updated when sheet structure is changed
  - transclusions are updated a resource is renamed
  - engine is updated when a resource is renamed (registered alias by name)

  We do not test the evaluation of transclusions here, which is done in `Engine.test.js`
*/
test('Transclusions: inserting a row', t => {
  let { archive } = _setup()
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  insertRows(sheetSession, 1, 1)
  let cell1 = article.get('cell1')
  t.equal(getSource(cell1), "'My Sheet'!A1:C4", "transclusion should have been updated")
  t.end()
})

test('Transclusions: deleting a row', t => {
  let { archive } = _setup()
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  deleteRows(sheetSession, 2, 1)
  let cell1 = article.get('cell1')
  t.equal(getSource(cell1), "'My Sheet'!A1:C2", "transclusion should have been updated")
  t.end()
})

test('Transclusions: inserting a column', t => {
  let { archive } = _setup()
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  insertCols(sheetSession, 1, 1)
  let cell1 = article.get('cell1')
  t.equal(getSource(cell1), "'My Sheet'!A1:D3", "transclusion should have been updated")
  t.end()
})

test('Transclusions: deleting a column', t => {
  let { archive } = _setup()
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  deleteCols(sheetSession, 1, 1)
  let cell1 = article.get('cell1')
  t.equal(getSource(cell1), "'My Sheet'!A1:B3", "transclusion should have been updated")
  t.end()
})

test('Transclusions: rename sheet', t => {
  let { archive } = _setup()
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  archive.renameDocument('sheet', 'Foo')
  let cell1 = article.get('cell1')
  t.equal(getSource(cell1), "'Foo'!A1:C3", "transclusion should have been updated")
  t.end()
})

function _setup() {
  let engine = new StubEngine()
  let context = { engine }
  let rawArchive = createRawArchive([
    {
      id: 'article',
      path: 'article.xml',
      type: 'article',
      name: 'My Article',
      body: [
        "<cell id='cell1' language='mini'>'My Sheet'!A1:C3</cell>"
      ]
    },
    {
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
  ])
  let archive = loadRawArchive(rawArchive, context)
  return { archive }
}

function loadRawArchive(rawArchive, context) {
  let archive = new StencilaArchive({}, {}, context)
  archive._sessions = archive._ingest(rawArchive)
  archive._upstreamArchive = rawArchive
  _initStencilaArchive(archive, context)
  return archive
}

// TODO: it should be easier stub out the engine
// ATM  the adapters heaviy use the engine's internal API to update
// the engine's internal model of these documents.
class StubEngine {
  run() {}
  addDocument() {
    return new StubEngineArticleModel()
  }
  addSheet() {
    return new StubEngineSheetModel()
  }
  _setResourceName() {}
  on() {}
}
class StubEngineArticleModel {
  setAutorun() {}
  updateCell() {}
}
class StubEngineSheetModel {
  insertRows() {}
  deleteRows() {}
  insertCols() {}
  deleteCols() {}
  updateCell() {}
}