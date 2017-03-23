import test from 'tape'
import MemoryBuffer from '../../src/backend/MemoryBuffer'
import TestStorer from '../backend/TestStorer'

import DocumentHTMLConverter from '../../src/document/DocumentHTMLConverter'
import testVFS from '../../tmp/test-vfs.js'
import wrapSnippet from '../../src/util/wrapSnippet'

let kitchenSinkHTML = wrapSnippet(testVFS['tests/documents/kitchen-sink/default.html'])

test('DocumentHTMLConverter:  Convert into internal buffer from an HTML file', function (t) {
  let converter = new DocumentHTMLConverter()
  let storer = _createFileStorer()
  let archive = new MemoryBuffer()

  converter.importDocument(
    storer,
    archive
  ).then((manifest) => {
    t.equal(manifest.type, 'document')
    t.equal(manifest.title, 'Kitchen sink')
    archive.readFile('index.html', 'text/html').then((html) => {
      t.equal(html, kitchenSinkHTML)
      t.end()
    })
  })
})

test('DocumentHTMLConverter: Convert to named HTML file from buffer', function (t) {
  let converter = new DocumentHTMLConverter()
  let archive = _createBuffer()
  let storer = new TestStorer('/path/to/archive', 'hello.html')
  converter.exportDocument(
    archive,
    storer
  ).then(() => {
    storer.readFile('hello.html', 'text/html').then((html) => {
      t.equal(html, kitchenSinkHTML)
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
  let storer = new TestStorer('/path/to/archive', 'hello.html')
  storer.writeFile('hello.html', 'text/html', kitchenSinkHTML)
  return storer
}

function _createBuffer() {
  let buffer = new MemoryBuffer()
  buffer.writeFile('index.html', 'text/html', kitchenSinkHTML)
  return buffer
}
