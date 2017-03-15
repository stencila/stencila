import test from 'tape'

import { isNil, EditorSession } from 'substance'
import { DocumentConfigurator, documentConversion, JsContext } from '../../index.es'
import CellEngine from '../../src/document/CellEngine'
import TestBackend from '../backend/TestBackend'

test('CellEngine: setup', (t) => {
  setupCellEngine()
  .then((cellEngine) => {
    t.notOk(isNil(cellEngine), 'Should have setup a CellEngine')
  })
})

function setupCellEngine() {
  return setupEditorSession('/tests/documents/simple/default.html')
  .then(({editorSession}) => {
    return new CellEngine(editorSession)
  })
}

function setupEditorSession(archiveURL) {
  let configurator = new DocumentConfigurator()
  let backend = new TestBackend()
  let archive = backend.getArchive(archiveURL)
  return new Promise((resolve, reject) => {
    archive.readFile('index.html').then((docHTML) => {
      let doc = documentConversion.importHTML(docHTML)
      let editorSession = new EditorSession(doc, {
        configurator: configurator,
        context: {
          stencilaContexts: {
            'js': new JsContext()
          }
        }
      })
      resolve({editorSession})
    }).catch((err) => {
      reject(err)
    })
  })
}