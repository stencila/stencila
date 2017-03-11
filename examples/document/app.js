/*
  WIP: a tiny integration of a Stencila Document editor
  using a set of stub services.
*/

import wrapSnippet from '../docs/wrapSnippet'
import example from '../docs/kitchensink'
import { EditorSession } from 'substance'
import { DocumentEditor, DocumentConfigurator, importHTML } from 'stencila-document'

let configurator = new DocumentConfigurator()
let doc = importHTML(wrapSnippet(example))
let editorSession = new EditorSession(doc, {
  configurator: configurator
})

window.addEventListener('load', () => {
  DocumentEditor.mount({
    editorSession,
    edit: true
  }, window.document.body)
})
