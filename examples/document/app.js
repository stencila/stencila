/*
  A tiny integration of a Stencila Document editor
  using a stub backend.
*/

import { DocumentPage, Host, MemoryBackend, getQueryStringParam } from 'stencila'

window.addEventListener('load', () => {
  window.documentPage = DocumentPage.mount({
    host: new Host({
      // Initial peers can be set in an environment variable
      peers: window.STENCILA_PEERS ? window.STENCILA_PEERS.split(' ') : [],
      // Peer discovery defaults to false but its frequency (in seconds) can be set in an environment variable
      discover: window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false,
    }),
    backend: new MemoryBackend(window.GUIDES),
    documentId: getQueryStringParam('documentId') || '01-welcome-to-stencila'
  }, window.document.body)

  function onKeyDown(e) {
    // CTRL+S
    if (e.ctrlKey && e.keyCode === 83) {
      console.info('saving')
      window.documentPage.save()
    }
  }
  document.addEventListener('keydown', onKeyDown, false)
})
