/*
  A tiny integration of a Stencila Document editor
  using a stub backend.
*/

import { EditorSession } from 'substance'
import {
  DatatableConfigurator, DatatablePage,
  DatatableDocument, MemoryDatastore, createSampleDatatable
} from 'stencila'

window.addEventListener('load', () => {
  let configurator = new DatatableConfigurator()

  // HACKING a sample together; this should be replaced
  // by something using an importer
  let sampleData = createSampleDatatable()
  let store = new MemoryDatastore(sampleData)
  let doc = new DatatableDocument(configurator.getSchema())
  doc.store = store
  let editorSession = new EditorSession(doc, { configurator })
  window.documentPage = DatatablePage.mount({
    editorSession
  }, window.document.body)
})
