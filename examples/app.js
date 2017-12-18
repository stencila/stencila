import {
  platform, substanceGlobals,
} from 'substance'
import {
  Host, Project, getQueryStringParam, FunctionManager, Engine, VirtualRDCLoader
} from 'stencila'

import fullup from './util/fullup'

let vfs = window.vfs

// Add a full up sheet dynamically
vfs.writeFileSync('examples/data/mini/fullup.xml', fullup().innerHTML)

// Note: this way we enable debug rendering only when
// devtools is open when this page is loaded
substanceGlobals.DEBUG_RENDERING = platform.devtools
console.info('USING DEBUG_RENDERING?', substanceGlobals.DEBUG_RENDERING)

window.addEventListener('load', () => {
  const example = getQueryStringParam('example') || 'mini'

  // Use virtual file system to construct document container
  let loader = new VirtualRDCLoader(vfs)
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
