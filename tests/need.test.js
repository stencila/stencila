const need = require('../src/need')

const test = require('tape')

test('need() can fetch a NPM module', t => {
  let isNumber = need('is-number')

  t.equal(isNumber(1), true)
  t.equal(isNumber('foo'), false)

  t.end()
})
