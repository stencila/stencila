/*
  WIP: a tiny integration of a Stencila Dashboard component
  using a set of stub services.
*/

import { MemoryBackend, Dashboard } from 'stencila'
let stubBackend = new MemoryBackend(window.GUIDES)

window.addEventListener('load', () => {
  Dashboard.mount({
    backend: stubBackend,
    resolveEditorURL: function(type, documentId) {
      let editorURL
      if (type === 'document') {
        editorURL = "../document/"
      } else {
        editorURL = "../sheet/"
      }
      editorURL += '?documentId='+encodeURIComponent(documentId)
      return editorURL
    }
  }, window.document.body)
})
