import { forEach, map } from 'substance'
import wrapSnippet from '../../examples/docs/wrapSnippet'
import { MemoryArchive } from '../../index.es'
import testVFS from '../../tmp/test-vfs.js'

let testLibrary = {}
forEach(testVFS, (entry, address) => {
  if (/\.html$/.exec(address)) {
    testLibrary[address] = {
      type: 'document',
      address,
      title: address,
      createdAt: '2017-03-10T00:03:12.060Z',
      modifiedAt: '2017-03-10T00:03:12.060Z',
      openedAt: '2017-03-10T00:03:12.060Z',
      _html: entry.data,
    }
  }
})

const SLASH = "/".charCodeAt(0)

export default class TestBackend {
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
    let archive = new MemoryArchive()
    archive.writeFile('index.html', 'text/html', wrapSnippet(entry._html))
    return archive
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
