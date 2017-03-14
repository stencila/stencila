import { forEach, map } from 'substance'
import wrapSnippet from '../../examples/docs/wrapSnippet'
import { MemoryArchive } from '../../index.es'
import testDocs from '../../tmp/test-docs.js'

let testLibrary = {}
forEach(testDocs, (entry, address) => {
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
    let entry = testLibrary[archiveURL]
    if (!entry) return
    let archive = new MemoryArchive()
    archive.writeFile('index.html', 'text/html', wrapSnippet(entry._html))
    return archive
  }
}
