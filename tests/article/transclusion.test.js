import test from 'tape'
import { insertRows, deleteRows, insertCols, deleteCols } from '../../src/sheet/sheetManipulations'
import { getSource } from '../../src/shared/cellHelpers'
import createRawArchive from '../util/createRawArchive'
import loadRawArchive from '../util/loadRawArchive'
import setupEngine from '../util/setupEngine'
import { play, getValues } from '../util/engineTestHelpers'
import { queryCells } from '../util/sheetTestHelpers'

/*
  Transclusions need to be updated whenever the referenced sheet changes
  structurally, i.e. rows or columns are added or removed, or the referenced
  resource is renamed.
  To test this behavior, an archive is created and manipulations as done by the
*/

test('Transclusions: inserting a row', t => {
  t.plan(1)
  let { archive, engine } = _setup(sample1())
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  play(engine)
  .then(() => {
    insertRows(sheetSession, 1, 1)
  })
  .then(() => {
    let cell1 = article.get('cell1')
    t.equal(getSource(cell1), "'My Sheet'!A1:C4", "transclusion should have been updated")
  })
})

test('Transclusions: deleting a row', t => {
  t.plan(1)
  let { archive, engine } = _setup(sample1())
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  play(engine)
  .then(() => {
    deleteRows(sheetSession, 2, 1)
  })
  .then(() => {
    let cell1 = article.get('cell1')
    t.equal(getSource(cell1), "'My Sheet'!A1:C2", "transclusion should have been updated")
  })
})

test('Transclusions: inserting a column', t => {
  t.plan(1)
  let { archive, engine } = _setup(sample1())
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  play(engine)
  .then(() => {
    insertCols(sheetSession, 1, 1)
  })
  .then(() => {
    let cell1 = article.get('cell1')
    t.equal(getSource(cell1), "'My Sheet'!A1:D3", "transclusion should have been updated")
  })
})

test('Transclusions: deleting a column', t => {
  t.plan(1)
  let { archive, engine } = _setup(sample1())
  let sheetSession = archive.getEditorSession('sheet')
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  play(engine)
  .then(() => {
    deleteCols(sheetSession, 1, 1)
  })
  .then(() => {
    let cell1 = article.get('cell1')
    t.equal(getSource(cell1), "'My Sheet'!A1:B3", "transclusion should have been updated")
  })
})

test('Transclusions: rename sheet', t => {
  t.plan(1)
  let { archive, engine } = _setup(sample1())
  let articleSession = archive.getEditorSession('article')
  let article = articleSession.getDocument()
  play(engine)
  .then(() => {
    archive.renameDocument('sheet', 'Foo')
  })
  .then(() => {
    let cell1 = article.get('cell1')
    t.equal(getSource(cell1), "'Foo'!A1:C3", "transclusion should have been updated")
  })
})

test('Transclusions: using a document variable in a sheet', t => {
  t.plan(1)
  let { engine } = _setup(sample2())
  let sheet = engine.getResource('sheet')
  play(engine)
  .then(() => {
    t.deepEqual(getValues(queryCells(sheet.cells, 'C1:C4')), [7, 6, 9, 22], 'transcluded values should have been computed correctly')
  })
})

test('Transclusions: rename document', t => {
  t.plan(2)
  let { archive, engine } = _setup(sample2())
  let sheet = engine.getResource('sheet')
  play(engine)
  .then(() => {
    archive.renameDocument('article', 'Foo')
  })
  .then(() => play(engine))
  .then(() => {
    t.equal(queryCells(sheet.cells, 'C1').source, "='Foo'!x", 'transclusion should have been updated')
    t.deepEqual(getValues(queryCells(sheet.cells, 'C1:C4')), [7, 6, 9, 22], 'transcluded values should have been computed correctly')
  })
})

function sample1() {
  return [
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
  ]
}

function sample2() {
  return [
    {
      id: 'article',
      path: 'article.xml',
      type: 'article',
      name: 'My Article',
      body: [
        "<cell id='cell1' language='mini'>x = 7</cell>"
      ]
    },
    {
      id: 'sheet',
      path: 'sheet.xml',
      type: 'sheet',
      name: 'My Sheet',
      columns: [{ name: 'x' }, { name: 'y' }, { name: 'z' }],
      cells: [
        ['1', '2', "='My Article'!x"],
        ['4', '5', '6'],
        ['7', '8', '9'],
        ['10', '11', '=sum(C1:C3)']
      ]
    }
  ]
}

function _setup(archiveData) {
  let { engine } = setupEngine()
  let context = { engine }
  let rawArchive = createRawArchive(archiveData)
  let archive = loadRawArchive(rawArchive, context)
  return { archive, engine }
}