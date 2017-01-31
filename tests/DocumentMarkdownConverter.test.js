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
  var content = '# markdown\n'
  converter.load(doc, content)
  t.equal(converter.dump(doc), content)
  t.end()
})

test('markdown code blocks', function (t) {
  const doc = new Document()
  const converter = new DocumentMarkdownConverter()

  var content = '```\nvar hmm = \'wat\'\n\n```\n'

  converter.load(doc, content)
  t.equal(converter.dump(doc), content)
  t.end()
})
