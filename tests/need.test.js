const need = require('../src/need')

const test = require('tape')

if (typeof window !== 'undefined') {
  test('need', t => {
    let isNumber = need('is-number')

    t.equal(isNumber(1), true)
    t.equal(isNumber('foo'), false)

    t.end()
  })
}

