import { forEach } from 'substance'
import wrapSnippet from '../../examples/docs/wrapSnippet'
import kitchenSink from '../../examples/docs/kitchensink'
import simpleSheet from '../../examples/docs/simple-sheet'
import MemoryArchive from './MemoryArchive'

/*
  NOTE: We know that MemoryArchive interally works synchronously, so we don't
        wait for the promise for seeding.
*/
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
  '/examples/kitchen-sink': {
    type: 'document',
    title: 'Kitchen Sink Document',
    createdAt: '2017-03-10T00:03:12.060Z',
    modifiedAt: '2017-03-10T00:03:12.060Z',
    openedAt: '2017-03-10T00:03:12.060Z',
    // just there to simulate the virtual file system
    __archive: kitchenSinkArchive
  },
  '/examples/simple-sheet': {
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
      forEach(LIBRARY_FIXTURE, (doc, address) => {
        documentEntries.push({
          type: doc.type,
          address: address,
          title: doc.title,
          openedAt: doc.openedAt,
          createAt: doc.modifiedAt,
          modifiedAt: doc.modifiedAt,
        })
      })
      resolve(documentEntries)
    })
  }

  getArchive(archiveURL) {
    // Returns an in-memory archive
    return LIBRARY_FIXTURE[archiveURL].__archive
    // Ideas for real persistent archives:
    // return new GithubArchive(archiveURL)
    // return new DatArchive(archiveURL)
    // return new LocalFolderArchive(archiveURL)
    // return new PostgresArchive(this.db, archiveURL)
  }

}
