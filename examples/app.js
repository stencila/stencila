import {
  platform, substanceGlobals,
} from 'substance'
import {
  createEntityDbSession
} from 'substance-texture'
import {
  Host, Project, getQueryStringParam, FunctionManager, Engine,
  ArticleLoader, SheetLoader
} from 'stencila'
import { VfsLoader } from 'rdc-js'


// TODO: This loader should be provided by Texture
const PubMetaLoader = {
  load(jsonStr) {
    return createEntityDbSession(jsonStr)
  }
}

const loaders = {
  'article': ArticleLoader,
  'sheet': SheetLoader,
  'pub-meta': PubMetaLoader
}

let vfs = window.vfs
let loader = new VfsLoader(vfs, loaders)

import fullup from './util/fullup'

// Add a full up sheet dynamically
vfs.writeFileSync('examples/data/mini/fullup.xml', fullup().innerHTML)

// Note: this way we enable debug rendering only when
// devtools is open when this page is loaded
substanceGlobals.DEBUG_RENDERING = platform.devtools
console.info('USING DEBUG_RENDERING?', substanceGlobals.DEBUG_RENDERING)

window.addEventListener('load', () => {
  const example = getQueryStringParam('example') || 'mini'

  // Use virtual file system to construct document container
  loader.load(`examples/data/${example}`).then(documentContainer => {

    let peers = (getQueryStringParam('peers') || window.STENCILA_PEERS)
    if (peers) peers = peers.split(',')

    const discover = window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false

    let functionManager = new FunctionManager()
    functionManager.importLibrary('core', window.STENCILA_LIBCORE)

    let host = new Host({
      functionManager,
      peers: peers,
      discover: discover,
    })
    host.initialize().then(() => {
      const engine = new Engine(host)
      new Project(null, {
        documentContainer,
        engine,
        functionManager
      }).mount(window.document.body)
    })
  })
})
