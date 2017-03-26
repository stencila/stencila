import { forEach, map } from 'substance'
import wrapSnippet from '../../src/util/wrapSnippet'
import { MemoryBuffer } from '../../index.es'
import { MemoryBackend } from '../../index.es'
import testVFS from '../../tmp/test-vfs.js'

let testLibrary = {}
let htmls = {}
let buffers = {}

forEach(testVFS, (content, documentId) => {
  // Only pick HTML documents
  if (/\.html$/.exec(documentId)) {
    testLibrary[documentId] = {
      id: documentId,
      type: 'document',
      title: documentId,
      storage: {
        storerType: "filesystem",
        contentType: "html",
        archivePath: "/path/to/archive",
        mainFilePath: "main.html"
      },
      createdAt: '2017-03-10T00:03:12.060Z',
      updatedAt: '2017-03-10T00:03:12.060Z',
    }
    htmls[documentId] = content
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
    let manifest = testLibrary[documentId]
    let html = htmls[documentId]
    if (!manifest) return

    if (buffers[documentId]) return buffers[documentId]
    let buffer = new MemoryBuffer()

    // NOTE: We know MemoryBuffer.writeFile is implemented synchronously so we
    // don't need to wait for the promise
    buffer.writeFile('index.html', 'text/html', wrapSnippet(html))
    buffer.writeFile(
      'stencila-manifest.json',
      'application/json',
      JSON.stringify(manifest, null, '  ')
    )
    return Promise.resolve(buffer)
  }

  _getEntries() {
    return map(testLibrary, (entry) => {
      return Object.assign({}, entry)
    })
  }

  _getEntry(documentId) {
    if (documentId.charCodeAt(0) === SLASH) documentId = documentId.slice(1)
    const entry = testLibrary[documentId]
    const content = htmls[documentId]
    if (!entry) throw new Error('TestBackend does not contain '+documentId)
    if (!content) throw new Error('No HTML content available for ', documentId)
    return Object.assign({}, entry, {content})
  }

  _getDocumentIds() {
    const entries = this._getEntries()
    const docUrls = entries.filter((entry) => {
      return entry.type === 'document'
    }).map((entry) => {
      return entry.id
    })
    return docUrls
  }
}
