const VegaContext = require('../../src/vega-context/VegaContext')

const test = require('tape')

test('VegaContext', t => {
  let c = new VegaContext()

  t.equal(c.constructor.name, 'VegaContext')
  t.ok(c instanceof VegaContext)

  t.end()
})

test('VegaContext.execute', t => {
  let c = new VegaContext()

  t.plan(3)

  c.execute().then(result => {
    t.deepEqual(result, { errors: null, output: null })
  })

  c.execute('').then(result => {
    t.deepEqual(result, { errors: null, output: null })
  })

  c.execute({}).then(result => {
    t.equal(result.output.value.substring(0, 106), '<svg class="marks" width="0" height="0" viewBox="0 0 0 0" version="1.1" xmlns="http://www.w3.org/2000/svg"')
  })
})

