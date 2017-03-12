import { forEach } from 'substance'
import wrapSnippet from '../../examples/docs/wrapSnippet'
import kitchenSink from '../../examples/docs/kitchensink'
import simpleSheet from '../../examples/docs/simple-sheet'

/*
  Same layout as the ~/.stencila/library.json file which is used to power
  Stencila Desktop. On the hub we may use a completely different layout
  stored in the database.
*/
const LIBRARY_FIXTURE = {
  '/examples/kitchen-sink': {
    type: 'document',
    source: 'source.html',
    title: 'Kitchen Sink Document',
    createdAt: '2017-03-10T00:03:12.060Z',
    modifiedAt: '2017-03-10T00:03:12.060Z',
    openedAt: '2017-03-10T00:03:12.060Z',
    // just there to simulate an HTML file on the file system
    __sourceHTML: wrapSnippet(kitchenSink),

  },
  '/examples/simple-sheet': {
    type: 'sheet',
    source: 'source.html',
    title: 'Simple Sheet',
    createdAt: '2017-03-12T00:03:12.060Z',
    modifiedAt: '2017-03-12T00:03:12.060Z',
    openedAt: '2017-03-12T00:03:12.060Z',
    // just there to simulate an HTML file on the file system
    __sourceHTML: wrapSnippet(simpleSheet)
  }
}

export default class BackendStub {
  /*
    Returns a list of document entries to power the dashboard UI

    Should be sorted by openedAt
  */
  listDocuments() {
    let documentEntries = []
    forEach(LIBRARY_FIXTURE, (doc, address) => {
      documentEntries.push({
        address: address,
        title: doc.title,
        openedAt: doc.openedAt,
        createAt: doc.modifiedAt,
        modifiedAt: doc.modifiedAt,
      })
    })
    return new Promise(function(resolve) {
      resolve(documentEntries)
    })
  }

  /*
    Read document from an address

    Returns a JSON containing the documen type (sheet|document|slide) and the
    raw HTML.

    Example:

    ```js
    getDocument('/examples/kitchen-sink') =>

    {
      type: 'document',
      html: '<html>...</html>'
    }
    ```
  */
  getDocument(address) {
    let libEntry = LIBRARY_FIXTURE[address]
    let result = {
      type: libEntry.type,
      html: libEntry.__sourceHTML
    }
    return new Promise(function(resolve) {
      resolve(result)
    })
  }

  /*
    Write document to an address
  */
  saveDocument(address, html) {
    LIBRARY_FIXTURE[address].__sourceHTML = html
    return new Promise(function(resolve) {
      resolve()
    })
  }
}
