import test from 'tape'
import MemoryBuffer from '../../src/backend/MemoryBuffer'
import DocumentHTMLConverter from '../../src/document/DocumentHTMLConverter'

test('DocumentHTMLConverter:  Convert into internal buffer from an HTML file', function (t) {
  let converter = new DocumentHTMLConverter()
  let storer = _createFileStorer()
  let archive = new MemoryBuffer()

  converter.importDocument(
    storer,
    archive,
    'hello.html'
  ).then((manifest) => {
    t.equal(manifest.type, 'document')
    archive.readFile('index.html', 'text/html').then((html) => {
      t.equal(html, 'HELLO WORLD')
      t.end()
    })
  })
})

test('DocumentHTMLConverter: Convert to named HTML file from buffer', function (t) {
  let converter = new DocumentHTMLConverter()
  let archive = _createBuffer()
  let storer = new MemoryBuffer()
  converter.exportDocument(
    archive,
    storer,
    'hello.html'
  ).then(() => {
    storer.readFile('hello.html', 'text/html').then((html) => {
      t.equal(html, 'HELLO WORLD')
      t.end()
    })
  })
})

test('DocumentHTMLConverter: Should match an HTML file name', function (t) {
  let matched = DocumentHTMLConverter.match('foo.html')
  t.ok(matched)
  t.end()
})

/*
  NOTE: We know that MemoryBuffer is implemented synchronously, so we don't
        wait for the promise.
*/
function _createFileStorer() {
  let storer = new MemoryBuffer()
  storer.writeFile('hello.html', 'text/html', 'HELLO WORLD')
  return storer
}

function _createBuffer() {
  let buffer = new MemoryBuffer()
  buffer.writeFile('index.html', 'text/html', 'HELLO WORLD')
  return buffer
}
