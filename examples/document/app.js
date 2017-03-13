/*
  A tiny integration of a Stencila Document editor
  using a stub backend.
*/

import { DocumentPage, MemoryBackend } from 'stencila'

window.addEventListener('load', () => {
  DocumentPage.mount({
    backend: new MemoryBackend(),
    archiveURL: '/examples/kitchen-sink'
  }, window.document.body)
})
