import test from 'tape'

import {render} from '../../../src/util/viz/vega'

test('vega.render', t => {
  t.plan(1)

  render({
    // TODO : use a minimal vega spec here
  }).then(svg => {
    t.equal(svg.substring(0, 106), '<svg class="marks" width="0" height="0" viewBox="0 0 0 0" version="1.1" xmlns="http://www.w3.org/2000/svg"')
  })
})
