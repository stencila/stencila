import test from 'tape'
import { RunCellCommand } from '../../src/article/ArticleEditorCommands'
import { _getSourceElement, getValue } from '../../src/shared/cellHelpers'
import createRawArchive from '../util/createRawArchive'
import loadRawArchive from '../util/loadRawArchive'
import setupEngine from '../util/setupEngine'
import { play } from '../util/engineTestHelpers'
import { _reset_rand } from '../contexts/libtest'


test('ArticleEditorCommands: RunCellCommand.getCommandState()', t => {
  let { archive } = _setup(sample())
  let editorSession = archive.getEditorSession('article')
  let commandState
  let article = editorSession.getDocument()
  let cell1 = article.get('cell1')
  let source1 = _getSourceElement(cell1)
  let containerId = cell1.parentNode.id
  // RunCellCommand should be enabled when cursor is inside source
  editorSession.setSelection({
    type: 'property',
    path: source1.getPath(),
    startOffset: 0
  })
  commandState = _getCommandState(editorSession, RunCellCommand.name)
  t.notOk(commandState.disabled, 'RunCellCommand should be enabled if cursor is inside of a cell')
  // RunCellCommand should be enabled when cell node is selected
  editorSession.setSelection({
    type: 'node',
    nodeId: cell1.id,
    containerId
  })
  commandState = _getCommandState(editorSession, RunCellCommand.name)
  t.notOk(commandState.disabled, 'RunCellCommand should be enabled if a cell node is selected entirely')
  // RunCellCommand should be disabled otherwise
  editorSession.setSelection(null)
  commandState = _getCommandState(editorSession, RunCellCommand.name)
  t.ok(commandState.disabled, 'RunCellCommand should be disabled when selection is null')

  editorSession.setSelection({
    type: 'property',
    path: ['p1', 'content'],
    startOffset: 0
  })
  commandState = _getCommandState(editorSession, RunCellCommand.name)
  t.ok(commandState.disabled, 'RunCellCommand should be disabled when cursor is inside another node')

  editorSession.setSelection({
    type: 'container',
    startPath: ['p1', 'content'],
    startOffset: 1,
    endPath: ['p2', 'content'],
    endOffset: 1,
    containerId
  })
  commandState = _getCommandState(editorSession, RunCellCommand.name)
  t.ok(commandState.disabled, 'RunCellCommand should be disabled when a range is selected spanning a cell (not a node selection)')

  t.end()
})

test('ArticleEditorCommands: RunCellCommand.execute()', t => {
  t.plan(1)
  _reset_rand()
  let { archive, engine } = _setup(sample(), true)
  let editorSession = archive.getEditorSession('article')
  let article = editorSession.getDocument()
  let cell1 = article.get('cell1')
  let source1 = _getSourceElement(cell1)
  play(engine)
  .then(() => {
    editorSession.setSelection({
      type: 'property',
      path: source1.getPath(),
      startOffset: 0
    })
    _executeCommand(editorSession, RunCellCommand.name)
  })
  .then(() => play(engine))
  .then(() => {
    let val = getValue(cell1)
    t.equal(val.data, 2, 'RunCellCommand.execute() should have triggered another evaluation of cell1')
  })
})


function sample() {
  return [
    {
      id: 'article',
      path: 'article.xml',
      type: 'article',
      name: 'My Article',
      body: [
        "<p id='p1'>...</p>",
        "<cell id='cell1' language='mini'>rand()</cell>",
        "<p id='p2'>...</p>",
      ]
    }
  ]
}

function _setup(archiveData, withEngine) {
  let host
  let engine
  let context = {}
  if (withEngine) {
    ({host, engine} = setupEngine())
    context.host = host
  }
  let rawArchive = createRawArchive(archiveData)
  let archive = loadRawArchive(rawArchive, context)
  return { archive, engine }
}

function _getCommandState(editorSession, name) {
  return editorSession._commandStates[name]
}

function _executeCommand(editorSession, name) {
  return editorSession.executeCommand(name)
}