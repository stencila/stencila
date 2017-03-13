/*
  WIP: a tiny integration of a Stencila Dashboard component
  using a set of stub services.
*/

import { MemoryBackend, Dashboard } from 'stencila'
let stubBackend = new MemoryBackend()

window.addEventListener('load', () => {
  Dashboard.mount({
    backend: stubBackend,
    resolveEditorURL: function(type, archiveURL) {
      let editorURL
      if (type === 'document') {
        editorURL = "../document/"
      } else {
        editorURL = "../sheet/"
      }
      editorURL += '?archiveURL='+encodeURIComponent(archiveURL)
      return editorURL
    }
  }, window.document.body)
})
