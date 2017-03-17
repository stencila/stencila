import test from 'tape'

import {render} from '../../../src/util/viz/vegaLite'

test('vegaLite.render', t => {
  t.plan(1)
  
  render({
    data: {
      values: [
        {type: 'A', height: 28},
        {type: 'B', height: 55},
        {type: 'C', height: 43}
      ]
    },
    mark: 'bar',
    encoding: {
      x: {field: 'type', type: 'ordinal'},
      y: {field: 'height', type: 'quantitative'}
    }
  }).then(svg => {
    t.equal(svg.substring(0, 18), '<svg class="marks"')
  })
})
