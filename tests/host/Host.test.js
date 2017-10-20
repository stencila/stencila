import test from 'tape'

import Host from '../../src/host/Host'

test('Host', t => {
  let h = new Host()

  t.ok(h instanceof Host)

  t.end()
})
