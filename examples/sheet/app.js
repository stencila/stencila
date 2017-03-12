/*
  WIP: a tiny integration of a Stencila Document editor
  using a set of stub services.
*/

import wrapSnippet from '../docs/wrapSnippet'
import example from '../docs/simple-sheet'
import { EditorSession } from 'substance'
import { SheetEditor, SheetConfigurator, sheetConversion } from 'stencila'

let configurator = new SheetConfigurator()
let doc = sheetConversion.importHTML(wrapSnippet(example))
let editorSession = new EditorSession(doc, {
  configurator: configurator
})

window.addEventListener('load', () => {
  window.doc = doc
  SheetEditor.mount({
    editorSession,
    edit: true
  }, window.document.body)
})
