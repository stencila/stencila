const Document = require('../src/Document')
const DocumentMarkdownConverter = require('../src/DocumentMarkdownConverter')

const test = require('tape')

test('DocumentMarkdownConverter can be constructed with options', t => {
  const converter = new DocumentMarkdownConverter()
  t.equal(typeof converter, 'object')
  t.equal(typeof converter.load, 'function')
  t.equal(typeof converter.dump, 'function')
  t.end()
})

test('load html from markdown with converter.load', function (t) {
  const doc = new Document()
  const converter = new DocumentMarkdownConverter()
  converter.load(doc, 'md', '# markdown')
  console.log(doc.dump('md'))
  t.end()
})

test('dump markdown from document html with converter.dump', function (t) {
  t.end()
})
