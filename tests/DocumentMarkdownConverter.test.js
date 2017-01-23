const fs = require('fs')

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
  var content = '# markdown'
  converter.load(doc, content)
  t.equal(converter.dump(doc), content)
  t.end()
})

test('markdown code blocks', function (t) {
  const doc = new Document()
  const converter = new DocumentMarkdownConverter()

  var content = '```\nvar hmm = \'wat\'\n```'

  converter.load(doc, content)
  t.ok(converter.dump(doc) === content)
  t.end()
})

test('documents/unordered-list', t => {
  const doc = new Document()

  fs.readFile('tests/documents/unordered-list/default.md', 'utf8', (err, md) => {
    t.notOk(err)

    doc.load(md, 'md')
    t.equal(doc.dump('md'), md)

    fs.readFile('tests/documents/unordered-list/alt-asterisk.md', 'utf8', (err, alt) => {
      t.notOk(err)
      doc.load(alt, 'md')
      t.equal(doc.dump('md'), md)
    })

    fs.readFile('tests/documents/unordered-list/alt-plus.md', 'utf8', (err, alt) => {
      t.notOk(err)
      doc.load(alt, 'md')
      t.equal(doc.dump('md'), md)
    })

    fs.readFile('tests/documents/unordered-list/default.html', 'utf8', (err, html) => {
      t.notOk(err)
      t.equal(doc.dump('html'), html)
    })

    t.end()
  })
})
