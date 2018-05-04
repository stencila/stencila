import test from 'tape'
import {
  getCellState, isExpression, getCellValue, getCellType, valueFromText,
  getSource, setSource, getError, getErrorMessage
} from '../../src/shared/cellHelpers'
import createRawArchive from '../util/createRawArchive'
import loadRawArchive from '../util/loadRawArchive'
import setupEngine from '../util/setupEngine'
import { play } from '../util/engineTestHelpers'


test('cellHelpers: getCellState', t => {
  let { archive } = _setup()
  let article = archive.getEditorSession('article').getDocument()
  let cell = article.get('cell1')
  let cellState = getCellState(cell)
  t.ok(Boolean(cellState), 'there should be a cell state')
  t.ok(Boolean(cellState.status), 'cell state should have a status')
  t.end()
})

test('cellHelpers: isExpression()', t => {
  t.ok(isExpression('= foo()'), 'a cell with leading "=" is considered an expression')
  t.end()
})

test('cellHelpers: getCellValue', t => {
  t.plan(1)
  let { archive, engine } = _setup()
  let article = archive.getEditorSession('article').getDocument()
  let cell = article.get('cell1')
  play(engine)
  .then(() => {
    t.deepEqual(getCellValue(cell), {type: "number", data: 1}, 'getCellValue() should provide an unpacked value')
  })
})

test('cellHelpers: getCellType', t => {
  // TODO: do we want cell types only in sheets?
  t.plan(2)
  let { archive, engine } = _setup(sheetSample())
  let sheet = archive.getEditorSession('sheet').getDocument()
  let cells = sheet.getCellMatrix()
  play(engine)
  .then(() => {
    t.deepEqual(getCellType(cells[0][0]), "number", 'getCellType() should provide the correct type')
    t.deepEqual(getCellType(cells[0][1]), "any", 'getCellType() should provide the correct type')
  })
})

test('cellHelpers: valueFromText', t => {
  // TODO: add more of thi
  t.deepEqual(valueFromText('false'), { type: "boolean", data: false }, 'valueFromText should provide a correct unpacked value')
  t.deepEqual(valueFromText('true'), { type: "boolean", data: true }, 'valueFromText should provide a correct unpacked value')
  t.deepEqual(valueFromText('1'), { type: "integer", data: 1 }, 'valueFromText should provide a correct unpacked value')
  t.deepEqual(valueFromText('1.2'), { type: "number", data: 1.2 }, 'valueFromText should provide a correct unpacked value')
  t.end()
})

test('cellHelpers: getSource', t => {
  let { archive } = _setup(sheetAndArticle())
  let article = archive.getEditorSession('article').getDocument()
  let sheet = archive.getEditorSession('sheet').getDocument()
  let articleCell = article.get('cell1')
  let sheetCell = sheet.get('cell1')
  t.equal(getSource(articleCell), '10', 'source of article cell should be provided correctly')
  t.equal(getSource(sheetCell), '1', 'source of sheet cell should be provided correctly')
  t.end()
})

test('cellHelpers: setSource', t => {
  let { archive } = _setup(sample())
  let article = archive.getEditorSession('article').getDocument()
  let cell = article.get('cell1')
  setSource(cell, '10')
  t.equal(getSource(cell), '10', 'source of cell should have been updated correctly')
  t.end()
})

test('cellHelpers: getErrorMessage', t => {
  // TODO: do we want cell types only in sheets?
  t.plan(6)
  let { archive, engine } = _setup([
    {
      id: 'article',
      path: 'article.xml',
      type: 'article',
      name: 'My Article',
      body: [
        "<cell id='cell1' language='mini' type='number'>u + v</cell>",
        "<cell id='cell2' language='mini'>x = y</cell>",
        "<cell id='cell3' language='mini'>y = x</cell>",
      ]
    }
  ])
  let article = archive.getEditorSession('article').getDocument()
  let cell1 = article.get('cell1')
  let cell2 = article.get('cell2')
  let cell3 = article.get('cell3')
  play(engine)
  .then(() => {
    _errorShouldContain(t, getErrorMessage(getError(cell1)), 'u', 'v')
    _errorShouldContain(t, getErrorMessage(getError(cell2)), 'x', 'y')
    _errorShouldContain(t, getErrorMessage(getError(cell3)), 'x', 'y')
  })
})

function _errorShouldContain(t, err, ...symbols) {
  for (let i = 0; i < symbols.length; i++) {
    let s = symbols[i]
    t.ok(err.indexOf(s)>=0, `error should contain ${s}`)
  }
}

function sample() {
  return [
    {
      id: 'article',
      path: 'article.xml',
      type: 'article',
      name: 'My Article',
      body: [
        "<p id='p1'>...</p>",
        "<cell id='cell1' language='mini' type='number'>1</cell>",
        "<p id='p2'>...</p>",
        "<cell id='cell2' language='mini'>2</cell>",
      ]
    }
  ]
}

function sheetSample() {
  return [{
    id: 'sheet',
    path: 'sheet.xml',
    type: 'sheet',
    name: 'My Sheet',
    columns: [{ name: 'x', type: 'number' }, { name: 'y' }, { name: 'z' }],
    cells: [
      ['1', '2', '3'],
      ['4', '5', '6'],
      ['7', '8', '9'],
      ['10', '11', '12']
    ]
  }]
}

function sheetAndArticle() {
  return [
    {
      id: 'article',
      path: 'article.xml',
      type: 'article',
      name: 'My Article',
      body: [
        "<cell id='cell1' language='mini' type='number'>10</cell>",
      ]
    },
    {
      id: 'sheet',
      path: 'sheet.xml',
      type: 'sheet',
      name: 'My Sheet',
      cells: [
        [{ id: 'cell1', source: '1'}, '2'],
        ['3', '4'],
      ]
    }
  ]
}

function _setup(archiveData) {
  archiveData = archiveData || sample()
  let host
  let engine
  let context = {}
  ;({host, engine} = setupEngine())
  context.host = host
  let rawArchive = createRawArchive(archiveData)
  let archive = loadRawArchive(rawArchive, context)
  return { archive, engine }
}
