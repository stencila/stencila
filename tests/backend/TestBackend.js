import { forEach, map } from 'substance'
import wrapSnippet from '../../examples/docs/wrapSnippet'
import { MemoryArchive } from '../../index.es'
import { MemoryBackend } from '../../index.es'
import testVFS from '../../tmp/test-vfs.js'

let testLibrary = {}
forEach(testVFS, (content, address) => {
  if (/\.html$/.exec(address)) {
    testLibrary[address] = {
      type: 'document',
      address,
      title: address,
      createdAt: '2017-03-10T00:03:12.060Z',
      modifiedAt: '2017-03-10T00:03:12.060Z',
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
  getArchive(archiveURL) {
    if (archiveURL.charCodeAt(0) === SLASH) {
      archiveURL = archiveURL.slice(1)
    }
    let entry = testLibrary[archiveURL]
    if (!entry) return
    if (entry._archive) return entry._archive
    let archive = new MemoryArchive()
    archive.writeFile('index.html', 'text/html', wrapSnippet(entry._html))
    entry._archive = Promise.resolve(archive)
    return entry._archive
  }

  _getEntries() {
    return map(testLibrary, (entry) => {
      return Object.assign({}, entry)
    })
  }

  _getDocumentUrls() {
    const entries = this._getEntries()
    const docUrls = entries.filter((entry) => {
      return entry.type === 'document'
    }).map((entry) => {
      return entry.address
    })
    return docUrls
  }
}
