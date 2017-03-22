import { forEach, map } from 'substance'
import wrapSnippet from '../../examples/docs/wrapSnippet'
import { MemoryBuffer } from '../../index.es'
import { MemoryBackend } from '../../index.es'
import testVFS from '../../tmp/test-vfs.js'

let testLibrary = {}
forEach(testVFS, (content, documentId) => {
  // Only pick HTML documents
  if (/\.html$/.exec(documentId)) {
    testLibrary[documentId] = {
      type: 'document',
      title: documentId,
      createdAt: '2017-03-10T00:03:12.060Z',
      updatedAt: '2017-03-10T00:03:12.060Z',
      _html: content,
    }
  }
})

const SLASH = "/".charCodeAt(0)

export default class TestBackend extends MemoryBackend {
  listDocuments() {
    return new Promise(function(resolve) {
      let documentEntries = map(testLibrary, (entry) => {
        return Object.assign({}, entry)
      })
      resolve(documentEntries)
    })
  }
  getBuffer(documentId) {
    if (documentId.charCodeAt(0) === SLASH) {
      documentId = documentId.slice(1)
    }
    let entry = testLibrary[documentId]
    if (!entry) return
    if (entry._buffer) return entry._buffer
    let buffer = new MemoryBuffer()
    buffer.writeFile('index.html', 'text/html', wrapSnippet(entry._html))
    entry._buffer = Promise.resolve(buffer)
    return entry._buffer
  }

  _getEntries() {
    return map(testLibrary, (entry) => {
      return Object.assign({}, entry)
    })
  }

  _getDocumentIds() {
    const entries = this._getEntries()
    const docUrls = entries.filter((entry) => {
      return entry.type === 'document'
    }).map((entry) => {
      return entry.address
    })
    return docUrls
  }
}
