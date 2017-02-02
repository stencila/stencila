const Document = require('../src/Document')

const test = require('tape')

test('Document can be constructed', t => {
  const doc = new Document()
  t.equal(typeof doc, 'object')
  t.equal(typeof doc.load, 'function')
  t.equal(typeof doc.dump, 'function')
  t.end()
})

test('Document content is exposed as a full property', function (t) {
  let d = new Document()

  t.equal(d.content, '')
  t.end()
})

test('Document can be loaded/dumped from/to HTML', function (t) {
  let d = new Document()

  t.equal(d.html, '')

  let html = '<p>Hello</p>'
  d.html = html
  t.equal(d.html, html)

  t.end()
})

test('Document can be loaded/dumped from/to Markdown', function (t) {
  let d = new Document()

  t.equal(d.md, '', 'md property is empty')
  t.equal(d.html, '', 'html property is empty')

  let md = 'Hello from *Markdown*!'
  let html = '<p>Hello from <em>Markdown</em>!</p>\n'

  d.load(md, 'md')
  t.equal(d.md, md, 'md content matches after loading markdown')
  t.equal(d.html, html, 'html content matches after loading markdown')

  d.load(html, 'html')
  t.equal(d.html, html, 'html content matches after loading html')
  t.equal(d.md, md, 'convert back to markdown')

  t.end()
})
