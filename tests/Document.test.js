const Document = require('../src/Document')

const test = require('tape')

test('Document can be constructed', t => {
  const doc = new Document()
  t.equal(typeof doc, 'object')
  t.equal(typeof doc.load, 'function')
  t.equal(typeof doc.dump, 'function')
  t.end()
})

test('load html from markdown with converter.load', t => {
  const doc = new Document()
  doc.load('md', '# markdown')
  console.log(doc.dump('md'))
  t.end()
})

test('dump markdown from document html with converter.dump', t => {
  t.end()
})
