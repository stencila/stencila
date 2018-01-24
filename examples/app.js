import {
  DocumentArchive, ManifestLoader,
  platform, substanceGlobals
} from 'substance'
import { PubMetaLoader } from 'substance-texture'

import {
  Project, getQueryStringParam,
  SheetLoader,
  setupStencilaContext,
  ArticleLoader
} from 'stencila'

import iceCreamSales from './util/ice-cream-sales'

// prepare the VFS on-the-fly expanding all examples
let vfs = window.vfs
// Add a sheet to dar dynamically
vfs.writeFileSync('examples/data/publication/ice-cream-sales.xml', iceCreamSales().innerHTML)



window.addEventListener('load', () => {
  // Note: this way we enable debug rendering only when
  // devtools is open when this page is loaded
  substanceGlobals.DEBUG_RENDERING = platform.devtools
  const example = getQueryStringParam('example') || 'mini'
  const discover = window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false
  const peers = getQueryStringParam('peers') || window.STENCILA_PEERS
  const libs = { 'core': window.STENCILA_LIBCORE }
  let documentArchive = _loadExamleDar(example, vfs)
  setupStencilaContext(documentArchive, {
    discover,
    peers,
    libs,
  }).then(({host, functionManager, engine}) => {
    new Project(null, {
      documentArchive,
      host,
      functionManager,
      engine
    }).mount(window.document.body)
  })
})


/*
  HACK: We can't use a regular loader pattern as pub-meta and manuscript
        depend on each other. Can we instead formulate dependencies in the
        loader configuration?
*/
function _loadExamleDar(example, vfs) {
  let manifestXML = vfs.readFileSync(`examples/data/${example}/manifest.xml`)
  let manifest = ManifestLoader.load(manifestXML)
  let sessions = {
    manifest: manifest.editorSession
  }
  let docs = manifest.manifest.findAll('documents > document')
  let pubMetaDbSession = PubMetaLoader.load()

  docs.forEach(entry => {
    let id = entry.attr('id')
    let type = entry.attr('type')
    let xml = vfs.readFileSync(`examples/data/${example}/${entry.path}`)
    if (type === 'article') {
      sessions[id] = ArticleLoader.load(xml, {
        pubMetaDb: pubMetaDbSession.getDocument()
      })
    } else if (type === 'sheet') {
      sessions[id] = SheetLoader.load(xml)
    }
  })

  sessions['pub-meta'] = pubMetaDbSession
  return new DocumentArchive(sessions)
}
