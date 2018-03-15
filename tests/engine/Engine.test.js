import test from 'tape'
import Engine from '../../src/engine/Engine'
import JsContext from '../../src/contexts/JsContext'
import MiniContext from '../../src/contexts/MiniContext'
import FunctionManager from '../../src/function/FunctionManager'
import { libtestXML, libtest } from '../contexts/libtest'
import { UNKNOWN } from '../../src/engine/CellStates'
// import { wait } from '../testHelpers'

test('Engine: [steps] single cell', t => {
  t.plan(9)
  let { engine, graph } = _setup()
  // this should automatically trigger code analysis and
  // incremental graph updates
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
  _cycle(engine)
  .then(() => {
    let nextActions = engine.getNextActions()
    t.equal(nextActions.size, 1, 'There should be one next action')
    let a = nextActions.get(id)
    t.equal(a.type, 'register', '.. which should a registration action')
    t.equal(cell.status, UNKNOWN, 'cell state should be UNKNOWN')
  })
  .then(() => _cycle(engine))
  .then(() => {
    t.ok(graph.hasCell(id), 'The cell should now be registered')
    let nextActions = engine.getNextActions()
    let a = nextActions.get(id)
    t.equal(a.type, 'evaluate', 'next action should be evaluate')
  })
  .then(() => _cycle(engine))
  .then(() => {
    let nextActions = engine.getNextActions()
    let a = nextActions.get(id)
    t.equal(a.type, 'update', 'next action should be update')
  })
  .then(() => _cycle(engine))
  .then(() => {
    let nextActions = engine.getNextActions()
    t.equal(nextActions.size, 0, 'There should be no pending actions')
    t.notOk(cell.hasErrors(), 'the cell should have no error')
    t.equal(_getValue(cell), 3, 'the value should have been computed correctly')
  })
})

test('Engine: [steps] sheet', t=> {
  t.plan(7)
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
  _cycle(engine)
  .then(() => {
    _checkActions(t, engine, [cell2, cell4], ['register', 'register'])
  })
  .then(() => {
    return _cycle(engine)
  })
  .then(() => {
    _checkActions(t, engine, [cell2, cell4], ['evaluate', 'evaluate'])
  })
  .then(() => {
    return _cycle(engine)
  })
  .then(() => {
    _checkActions(t, engine, [cell2, cell4], ['update', 'update'])
  })
  .then(() => {
    return _cycle(engine)
  })
  .then(() => {
    t.deepEqual(_getValues([cell2, cell4]), [2,4], 'values should have been computed')
  })
})

test('Engine: [play] sheet', t=> {
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
  _play(engine)
  .then(() => {
    t.deepEqual(_getValues(sheet.getCells('B1:B2')), [2,4], 'values should have been computed')
  })
})

test('Engine: [play] simple doc', t => {
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
  _play(engine)
  .then(() => {
    t.deepEqual(_getValues(cells), [2,3,5], 'values should have been computed')
  })
})


/*
  Waits for all actions to be finished.
  This is the slowest kind of scheduling, as every cycle
  takes as long as the longest evaluation.
  In a real environment, the Engine should be triggered as often as possible,
  but still with a little delay, so that all 'simultanous' actions can be
  done at once.
*/
function _cycle(engine) {
  let actions = engine.cycle()
  return Promise.all(actions)
}

/*
  Triggers a cycle as long as next actions are coming in.
*/
function _play(engine) {
  return new Promise((resolve) => {
    function step() {
      if (engine.needsUpdate()) {
        _cycle(engine).then(step)
      } else {
        resolve()
      }
    }
    step()
  })
}

function _checkActions(t, engine, cells, actions) {
  let nextActions = engine.getNextActions()
  for (let i = 0; i < cells.length; i++) {
    const cell = cells[i]
    const action = nextActions.get(cell.id)
    const actual = action ? action.type : undefined
    const expected = actions[i]
    t.equal(actual, expected, 'action should be registered correctly')
  }
}

// TODO: there must be a helper, already
// look into other tests
function _getValue(cell) {
  if (cell.value) {
    return cell.value.data
  }
}

function _getValues(cells) {
  return cells.map(cell => _getValue(cell))
}

function _setup() {
  // A JsContext with the test function library
  let jsContext = new JsContext()
  let miniContext
  jsContext.importLibrary('test', libtest)
  // Function manager for getting function specs
  let functionManager = new FunctionManager()
  functionManager.importLibrary('test', libtestXML)
  // A mock Host that provides the JsContext when requested
  let host = {
    createContext: function(lang) {
      switch (lang) {
        case 'js':
          return Promise.resolve(jsContext)
        case 'mini':
          return Promise.resolve(miniContext)
        default:
          return Promise.reject(new Error('No context for language '+lang))
      }
    },
    functionManager
  }
  miniContext = new MiniContext(host)
  let engine = new Engine(host)
  let graph = engine._graph
  return { host, engine, graph }
}