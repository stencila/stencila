import test from 'tape'

import Host from '../../src/host/Host'
import Engine from '../../src/engine/Engine'
import { wait } from '../testHelpers'

/*
  Next:
  - Find a concept for dealing with the async nature of the engine API
*/
test('Engine: simple cell', t => {
  t.plan(1)
  let { engine } = _setup()
  // this should automatically trigger code analysis and
  // incremental graph updates
  engine.addCell({
    id: 'sheet1.cell1',
    lang: 'mini',
    source: '1+2',
    docId: 'sheet1'
  })
  // At any time the engine will have a number of pending actions
  // such as updating the cell graph or triggering evaluations, like a job queue.
  // Instead of processing the job queue automatically we could use a controllable
  // scheduler, that we can trigger manually for testing.
  // For real execution, we would apply an automated version, e.g. using a
  // stop-and-go strategy, where the job queue is processed periodically (e.g. every 50ms).
  wait(50)
  .then(() => {
    // TODO: in general it will not be necessary to
    // wait for evaluations. For testing it would be nice,
    // but we could also just wait for it.
    return engine.step()
  })
  .then(() => {
    debugger
    t.fail('NOT IMPLEMENTED YET')
  })
})

function _setup() {
  let host = new Host()
  let engine = new Engine(host)
  return { host, engine }
}