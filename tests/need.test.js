const need = require('../src/need')

const test = require('tape')

test.skip('need() can fetch a NPM module', t => {
  let isNumber = need('is-number')

  t.equal(isNumber(1), true)
  t.equal(isNumber('foo'), false)

  let isNumberCached = need('is-number')
  t.deepEqual(isNumber, isNumberCached)

  t.throws(() => need('a-non-existent-npm-module'))

  t.end()
})
