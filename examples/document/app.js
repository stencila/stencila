/*
  A tiny integration of a Stencila Document editor
  using a stub backend.
*/

import { DocumentPage, MemoryBackend, getQueryStringParam } from 'stencila'

console.info('Loading...', getQueryStringParam('documentId'))
window.addEventListener('load', () => {
  DocumentPage.mount({
    backend: new MemoryBackend(),
    documentId: getQueryStringParam('documentId') || 'kitchen-sink'
  }, window.document.body)
})
