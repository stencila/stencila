import { forEach } from 'substance'
import MemoryBuffer from './MemoryBuffer'

function _idToName(id) {
  let str = id.slice(3)
  str = str.charAt(0).toUpperCase() + str.slice(1)
  return str.replace(/-/g, ' ')
}

/*
  Creates a manifest for a given documentId
*/
function _getManifest(documentId) {
  return {
    type: 'document',
    title: _idToName(documentId),
    createdAt: '2017-03-10T00:03:12.060Z',
    updatedAt: '2017-03-10T00:03:12.060Z',
    storage: {
      storerType: "filesystem",
      contentType: "html",
      archivePath: "/some/path/kitchen-sink/storage",
      mainFilePath: "index.html"
    }
  }
}

export default class MemoryBackend {
  /*
    Takes an object with documentIds and HTML content
  */
  constructor(documents) {
    this.documents = documents
  }

  /*
    Returns a list of document entries to power the dashboard UI

    Should be sorted by openedAt
  */
  listDocuments() {
    return new Promise((resolve) => {
      let documentEntries = []
      forEach(this.documents, (html, documentId) => {
        let manifest = _getManifest(documentId)
        let entry = Object.assign({}, manifest, {id: documentId})
        documentEntries.push(entry)
      })
      resolve(documentEntries)
    })
  }

  deleteDocument(documentId) {
    delete this.documents[documentId]
    return Promise.resolve(this)
  }

  /*
    Returns a buffer object.

    Use MemoryBuffer implementation as an API reference
  */
  getBuffer(documentId) {
    let buffer = new MemoryBuffer()
    let html = this.documents[documentId]
    let manifest = _getManifest(documentId)
    // NOTE: We know that MemoryBuffer interally works synchronously, so we don't
    //       wait for the promise for seeding.
    buffer.writeFile('index.html', 'text/html', html)
    buffer.writeFile('stencila-manifest.json', 'application/json', JSON.stringify(manifest, null, ' '))
    return Promise.resolve(buffer)
  }

  storeBuffer(/*buffer*/) {
    return Promise.resolve()
  }

  updateManifest(/* documentId, props */) {
    return Promise.resolve()
  }

}
