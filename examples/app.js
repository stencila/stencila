import {
  platform, substanceGlobals,
} from 'substance'
import {
  createEntityDbSession
} from 'substance-texture'
import {
  Project, getQueryStringParam,
  ArticleLoader, SheetLoader,
  setupStencilaContext
} from 'stencila'
import { VfsLoader } from 'rdc-js'
import fullup from './util/fullup'

// TODO: This loader should be provided by Texture
const PubMetaLoader = {
  load(jsonStr) {
    return createEntityDbSession(jsonStr)
  }
}

window.addEventListener('load', () => {
  // Note: this way we enable debug rendering only when
  // devtools is open when this page is loaded
  substanceGlobals.DEBUG_RENDERING = platform.devtools

  const example = getQueryStringParam('example') || 'mini'
  const discover = window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false
  const peers = getQueryStringParam('peers') || window.STENCILA_PEERS
  const libs = { 'core': window.STENCILA_LIBCORE }

  // prepare the VFS on-the-fly expanding all examples
  let vfs = window.vfs
  // Add a full up sheet dynamically
  vfs.writeFileSync('examples/data/mini/fullup.xml', fullup().innerHTML)
  const loaders = {
    'article': ArticleLoader,
    'sheet': SheetLoader,
    'pub-meta': PubMetaLoader
  }
  // Use virtual file system to construct document container
  let loader = new VfsLoader(vfs, loaders)
  loader.load(`examples/data/${example}`).then(documentContainer => {
    const { host, functionManager, engine } = setupStencilaContext(documentContainer, {
      discover,
      peers,
      libs,
    })
    host.initialize().then(() => {
      new Project(null, {
        documentContainer,
        host,
        functionManager,
        engine
      }).mount(window.document.body)
    })
  })
})
