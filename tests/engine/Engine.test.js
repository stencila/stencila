import test from 'tape'

import Host from '../../src/host/Host'
import Engine from '../../src/engine/Engine'
import { wait } from '../testHelpers'

/*
  Next:
  - Find a concept for dealing with the async nature of the engine API
*/
test('Engine: simple cell', t => {
  let { engine } = _setup()
  // this should automatically trigger code analysis and
  // incremental graph updates
  engine.addCell({
    id: 'sheet1.cell1',
    lang: 'mini',
    source: '1+2',
    docId: 'sheet1'
  })

  // wait for all actions to be finished
  debugger
  engine.awaitCycle()
  t.fail('WOOO')
  t.end()
})

function _setup() {
  let host = new Host()
  let engine = new Engine(host)
  return { host, engine }
}