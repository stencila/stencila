/*
  WIP: a tiny integration of a Stencila Document editor
  using a set of stub services.
*/

import wrapSnippet from '../docs/wrapSnippet'
import example from '../docs/kitchensink'
import { EditorSession } from 'substance'
import { DocumentEditor, DocumentConfigurator, documentConversion, JsContext } from 'stencila'

let configurator = new DocumentConfigurator()
let doc = documentConversion.importHTML(wrapSnippet(example))
let editorSession = new EditorSession(doc, {
  configurator: configurator,
  context: {
    stencilaContexts: {
      'js': new JsContext()
    }
  }
})

window.addEventListener('load', () => {
  DocumentEditor.mount({
    editorSession,
    edit: true
  }, window.document.body)
})
