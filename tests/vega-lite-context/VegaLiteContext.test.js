const VegaLiteContext = require('../../src/vega-lite-context/VegaLiteContext')

const test = require('tape')

test('VegaLiteContext', t => {
  let c = new VegaLiteContext()

  t.equal(c.constructor.name, 'VegaLiteContext')
  t.ok(c instanceof VegaLiteContext)

  t.end()
})

test('VegaLiteContext.execute', t => {
  let c = new VegaLiteContext()

  t.plan(4)

  c.execute().then(result => {
    t.deepEqual(result, { errors: null, output: null })
  })

  c.execute('').then(result => {
    t.deepEqual(result, { errors: null, output: null })
  })

  c.execute({
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
  }).then(result => {
    t.equal(result.output.value.substring(0, 18), '<svg class="marks"')
  })

  c.execute(`
  let data = [
    {type: 'A', height: 28},
    {type: 'B', height: 55},
    {type: 'C', height: 43}
  ]
  return {
    data: {
      values: data
    },
    mark: 'bar',
    encoding: {
      x: {field: 'type', type: 'ordinal'},
      y: {field: 'height', type: 'quantitative'}
    }
  }`).then(result => {
    t.equal(result.output.value.substring(0, 18), '<svg class="marks"')
  })
})

