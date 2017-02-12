const test = require('tape')

const ComponentHttpStorer = require('../../src/component/ComponentHttpStorer')
const errors = require('../../src/component/component-storer-errors')

test('ComponentHttpStorer.read', t => {
  let s = new ComponentHttpStorer()
  s.read('http://raw.githubusercontent.com/stencila/test/master/README.md')
    .then(content => {
      t.equal(content.substring(0, 11), '![Stencila]')
      t.end()
    })
    .catch(error => {
      t.notOk(error)
      t.end()
    })
})

test('ComponentHttpStorer.read HTTPS', t => {
  let s = new ComponentHttpStorer()
  s.read('https://raw.githubusercontent.com/stencila/test/master/README.md')
    .then(content => {
      t.equal(content.substring(0, 11), '![Stencila]')
      t.end()
    })
    .catch(error => {
      t.notOk(error)
      t.end()
    })
})

test('ComponentHttpStorer.read nonexistant domain', t => {
  let s = new ComponentHttpStorer()
  s.read('http://doe5-n0t-ex1sts.com')
    .then(content => {
      t.fail('should not get here')
      t.end()
    })
    .catch(error => {
      t.ok(error instanceof errors.ComponentStorerUnfound)
      t.end()
    })
})

test('ComponentHttpStorer.read nonexistant path', t => {
  let s = new ComponentHttpStorer()
  s.read('http://httpbin.org/doe5-n0t-ex1sts')
    .then(content => {
      t.fail('should not get here')
      t.end()
    })
    .catch(error => {
      t.ok(error instanceof errors.ComponentStorerUnfound)
      t.end()
    })
})

test('ComponentHttpStorer.write', t => {
  let s = new ComponentHttpStorer()
  s.write('http://foo/bar')
    .then(content => {
      t.fail('should not get here')
      t.end()
    })
    .catch(error => {
      t.ok(error instanceof errors.ComponentStorerUnwritable)
      t.end()
    })
})

