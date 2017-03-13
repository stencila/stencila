/*
  A tiny integration of a Stencila Document editor
  using a stub backend.
*/

import { DocumentPage, MemoryBackend, getQueryStringParam } from 'stencila'

console.info('Loading...', getQueryStringParam('archiveURL'))
window.addEventListener('load', () => {
  DocumentPage.mount({
    backend: new MemoryBackend(),
    archiveURL: getQueryStringParam('archiveURL') || '/examples/kitchen-sink'
  }, window.document.body)
})
