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
import geneErrors from './util/gene-errors'
import geneData from './util/gene-data'
import rCells from './util/r-cells'

// prepare the VFS on-the-fly expanding all examples
let vfs = window.vfs
// Add a sheet to dar dynamically
vfs.writeFileSync('examples/data/publication/ice-cream-sales.xml', iceCreamSales().innerHTML)
vfs.writeFileSync('examples/data/gene-data/sheet.xml', geneData().innerHTML)
vfs.writeFileSync('examples/data/gene-errors/sheet.xml', geneErrors().innerHTML)
vfs.writeFileSync('examples/data/r-cells/sheet.xml', rCells().innerHTML)

window.addEventListener('load', () => {
  // Note: this way we enable debug rendering only when
  // devtools is open when this page is loaded
  substanceGlobals.DEBUG_RENDERING = platform.devtools
  const example = getQueryStringParam('example') || 'mini'

  let documentArchive = _loadExamleDar(example, vfs)
  setupStencilaContext(documentArchive).then(({host, functionManager, engine}) => {
    // Added for easier inspection of host during development
    window.host = host
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
