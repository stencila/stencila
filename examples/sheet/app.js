import { Configurator, EditorSession } from 'substance'
import {
  SheetPackage, SheetEditor, SheetSchema, Host, getQueryStringParam,
  FunctionManager, Engine, SheetEngineAdapter
} from 'stencila'

import blank from './blank'
import rCells from './r-cells'
import rModel from './r-model'
import viewModes from './view-modes'
import geneData from './gene-data'
import geneErrors from './gene-errors'
import fullup from './fullup'

const EXAMPLES = {
  'blank': blank,
  'r-cells': rCells,
  'r-model': rModel,
  'view-modes': viewModes,
  'gene-data': geneData,
  'gene-errors': geneErrors,
  'fullup': fullup
}

window.addEventListener('load', () => {
  const example = getQueryStringParam('example') || 'blank'

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
    let configurator = new Configurator()
    configurator.import(SheetPackage)
    const importer = configurator.createImporter(SheetSchema.getName())

    let generator = EXAMPLES[example]
    if (!generator) console.error('No such example: ' + example)
    const xml = generator()
    const sheet = importer.importDocument(xml)

    const editorSession = new EditorSession(sheet, {
      configurator: configurator,
      context: {
        host
      }
    })

    // editorSession.issueManager = editorSession.getManager('issue-manager')
    const engine = new Engine(host)
    let engineAdapter = new SheetEngineAdapter(editorSession)
    engineAdapter.connect(engine)
    engine.editorSession = editorSession

    new SheetEditor(null, {
      editorSession
    }, {
      context: {
        // editorSession,
        host,
        engine
      }
    }).mount(window.document.body)
  })
})
