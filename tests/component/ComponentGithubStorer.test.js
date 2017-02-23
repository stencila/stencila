const test = require('tape')

const ComponentGithubStorer = require('../../src/component/ComponentGithubStorer')
const errors = require('../../src/component/component-storer-errors')

test('ComponentGithubStorer.read', t => {
  let s = new ComponentGithubStorer()
  s.read('gh://stencila/test/README.md')
    .then(content => {
      t.equal(content.substring(0, 11), '![Stencila]')
      t.end()
    })
    .catch(error => {
      t.notOk(error)
      t.end()
    })
})

test('ComponentGithubStorer.read version', t => {
  let s = new ComponentGithubStorer()
  s.read('gh://stencila/test/README.md@64cbc155d5468a721fc1b186c45a141aae744679')
    .then(content => {
      t.equal(content.substring(0, 4), 'test')
      t.end()
    })
    .catch(error => {
      t.notOk(error)
      t.end()
    })
})

test('ComponentGithubStorer.read malformed repo', t => {
  let s = new ComponentGithubStorer()
  s.read('gh://mal-formed')
    .then(content => {
      t.fail('should not get here')
      t.end()
    })
    .catch(error => {
      t.ok(error instanceof errors.ComponentStorerMalformed)
      t.end()
    })
})

test('ComponentGithubStorer.read nonexistant repo', t => {
  let s = new ComponentGithubStorer()
  s.read('gh://d0es/nt/ex1st')
    .then(content => {
      t.fail('should not get here')
      t.end()
    })
    .catch(error => {
      t.ok(error instanceof errors.ComponentStorerUnfound)
      t.end()
    })
})

test.skip('ComponentGithubStorer.write', t => {
  let s = new ComponentGithubStorer()
  // Login as a user with access to this repo to run this test
  s.login('TOKEN')
    .write('gh://stencila/test/document.md', 'Hello world')
    .then(() => {
      t.end()
    })
    .catch(error => {
      t.notOk(error)
      t.end()
    })
})

test('ComponentGithubStorer.write unauthenticated', t => {
  let s = new ComponentGithubStorer()
  s.write('gh://stencila/test/document.md', 'Hello world')
    .then(() => {
      t.fail('should not get here')
      t.end()
    })
    .catch(error => {
      // Github returns a 404 if not authenticated
      t.ok(error instanceof errors.ComponentStorerUnfound)
      t.end()
    })
})
