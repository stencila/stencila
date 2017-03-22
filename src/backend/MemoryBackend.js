import { forEach } from 'substance'
import wrapSnippet from '../util/wrapSnippet'
import kitchenSink from '../../examples/docs/kitchensink'
import stencilaIntro from '../../examples/docs/stencila-intro'
import simpleSheet from '../../examples/docs/simple-sheet'
import guideMini from '../../docs/guide/mini.js'
import MemoryBuffer from './MemoryBuffer'

let stencilaIntroManifest = {
  type: 'document',
  title: 'Welcome to Stencila',
  createdAt: '2017-03-10T00:03:12.060Z',
  updatedAt: '2017-03-10T00:03:12.060Z',
  storage: {
    storerType: "filesystem",
    contentType: "html",
    archivePath: "/Users/john/Desktop",
    mainFilePath: "welcome-to-stencila.html",
    external: true
  }
}

let kitchenSinkManifest = {
  type: 'document',
  title: 'Kitchen Sink Document',
  createdAt: '2017-03-10T00:03:12.060Z',
  updatedAt: '2017-03-10T00:03:12.060Z',
  storage: {
    storerType: "filesystem",
    contentType: "html",
    archivePath: "/Users/john/Documents/Stencila/kitchen-sink/storage",
    mainFilePath: "index.html"
  }
}

let simpleSheetManifest = {
  type: 'sheet',
  title: 'Simple Sheet',
  createdAt: '2017-03-12T00:03:12.060Z',
  updatedAt: '2017-03-12T00:03:12.060Z',
  storage: {
    storerType: "filesystem",
    contentType: "html",
    archivePath: "/Users/john/Documents/Stencila/simple-sheet/storage",
    mainFilePath: "index.html"
  }
}

let guideMiniManifest = {
  type: 'document',
  title: 'A guide to Mini',
  createdAt: '2017-03-12T00:03:12.060Z',
  updatedAt: '2017-03-12T00:03:12.060Z',
  storage: {
    storerType: "filesystem",
    contentType: "html",
    archivePath: "/Users/john/Documents/Stencila/guide-mini/storage",
    mainFilePath: "index.html"
  }
}

/*
  NOTE: We know that MemoryBuffer interally works synchronously, so we don't
        wait for the promise for seeding.
*/
let stencilaIntroBuffer = new MemoryBuffer()
stencilaIntroBuffer.writeFile('index.html', 'text/html', wrapSnippet(stencilaIntro))
stencilaIntroBuffer.writeFile('stencila-manifest.json', 'application/json', JSON.stringify(stencilaIntroManifest, null, ' '))

let kitchenSinkBuffer = new MemoryBuffer()
kitchenSinkBuffer.writeFile('index.html', 'text/html', wrapSnippet(kitchenSink))
kitchenSinkBuffer.writeFile('stencila-manifest.json', 'application/json', JSON.stringify(kitchenSinkManifest, null, ' '))

let simpleSheetBuffer = new MemoryBuffer()
simpleSheetBuffer.writeFile('index.html', 'text/html', wrapSnippet(simpleSheet))
simpleSheetBuffer.writeFile('stencila-manifest.json', 'application/json', JSON.stringify(simpleSheetManifest, null, ' '))

let guideMiniBuffer = new MemoryBuffer()
guideMiniBuffer.writeFile('index.html', 'text/html', wrapSnippet(guideMini))
guideMiniBuffer.writeFile('stencila-manifest.json', 'application/json', JSON.stringify(guideMiniManifest, null, ' '))

/*
  Same layout as the ~/Documents/Stencila/library.json file which is used to power
  Stencila Desktop. On the hub we may use a completely different layout
  stored in the database.
*/

let LIBRARY_FIXTURE = {
  'stencila-intro': stencilaIntroManifest,
  'kitchen-sink': kitchenSinkManifest,
  'simple-sheet': simpleSheetManifest,
  'guide-mini': guideMiniManifest
}

let BUFFERS_FIXTURE = {
  'stencila-intro': stencilaIntroBuffer,
  'kitchen-sink': kitchenSinkBuffer,
  'simple-sheet': simpleSheetBuffer,
  'guide-mini': guideMiniBuffer
}


export default class BackendStub {

  /*
    Returns a list of document entries to power the dashboard UI

    Should be sorted by openedAt
  */
  listDocuments() {
    return new Promise(function(resolve) {
      let documentEntries = []
      forEach(LIBRARY_FIXTURE, (doc, documentId) => {
        let entry = Object.assign({}, doc, {id: documentId})
        documentEntries.push(entry)
      })
      resolve(documentEntries)
    })
  }

  deleteDocument(documentId) {
    delete LIBRARY_FIXTURE[documentId]
    return Promise.resolve(this)
  }

  /*
    Returns a buffer object.

    Use MemoryBuffer implementation as an API reference
  */
  getBuffer(documentId) {
    return Promise.resolve(BUFFERS_FIXTURE[documentId])
  }

  storeBuffer(/*buffer*/) {
    return Promise.resolve()
  }

  updateManifest(/* documentId, props */) {
    return Promise.resolve()
  }

}
