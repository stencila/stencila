import { forEach } from 'substance'
import wrapSnippet from '../util/wrapSnippet'
import kitchenSink from '../../examples/docs/kitchensink'
import stencilaIntro from '../../examples/docs/stencila-intro'
import simpleSheet from '../../examples/docs/simple-sheet'
import MemoryArchive from './MemoryArchive'

/*
  NOTE: We know that MemoryArchive interally works synchronously, so we don't
        wait for the promise for seeding.
*/
let stencilaIntroArchive = new MemoryArchive()
stencilaIntroArchive.writeFile('index.html', 'text/html', wrapSnippet(stencilaIntro))

let kitchenSinkArchive = new MemoryArchive()
kitchenSinkArchive.writeFile('index.html', 'text/html', wrapSnippet(kitchenSink))

let simpleSheetArchive = new MemoryArchive()
simpleSheetArchive.writeFile('index.html', 'text/html', wrapSnippet(simpleSheet))

/*
  Same layout as the ~/.stencila/library.json file which is used to power
  Stencila Desktop. On the hub we may use a completely different layout
  stored in the database.
*/
const LIBRARY_FIXTURE = {
  'stencila-intro': {
    type: 'document',
    title: 'Welcome to Stencila',
    createdAt: '2017-03-10T00:03:12.060Z',
    modifiedAt: '2017-03-10T00:03:12.060Z',
    openedAt: '2017-03-10T00:03:12.060Z',
    // just there to simulate the virtual file system
    __archive: stencilaIntroArchive
  },
  'kitchen-sink': {
    type: 'document',
    title: 'Kitchen Sink Document',
    createdAt: '2017-03-10T00:03:12.060Z',
    modifiedAt: '2017-03-10T00:03:12.060Z',
    openedAt: '2017-03-10T00:03:12.060Z',
    // just there to simulate the virtual file system
    __archive: kitchenSinkArchive
  },
  'simple-sheet': {
    type: 'sheet',
    source: 'source.html',
    title: 'Simple Sheet',
    createdAt: '2017-03-12T00:03:12.060Z',
    modifiedAt: '2017-03-12T00:03:12.060Z',
    openedAt: '2017-03-12T00:03:12.060Z',
    // just there to simulate an HTML file on the file system
    __archive: simpleSheetArchive
  }
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
        documentEntries.push({
          id: documentId,
          type: doc.type,
          address: documentId,
          title: doc.title,
          openedAt: doc.openedAt,
          createAt: doc.modifiedAt,
          modifiedAt: doc.modifiedAt,
        })
      })
      resolve(documentEntries)
    })
  }

  /*
    Returns an archive object.

    Use MemoryArchive implementation as an API reference
  */
  getArchive(documentId) {
    return new Promise(function(resolve) {
      resolve(LIBRARY_FIXTURE[documentId].__archive)
    })
  }

  storeArchive(/*archive*/) {
    return Promise.resolve()
  }

  updateManifest(/* documentId, props */) {
    return Promise.resolve()
  }

}
