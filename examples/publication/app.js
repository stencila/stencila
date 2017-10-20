/*
  A tiny integration of a Stencila Publication editor
  using a stub backend.
*/

import { Publication, Host, MemoryBackend, getQueryStringParam, FunctionManager } from 'stencila'

window.addEventListener('load', () => {
  let functionManager = new FunctionManager()
  functionManager.importLibrary('core', window.STENCILA_LIBCORE)

  window.pub = Publication.mount({
    host: new Host({
      functionManager,
      // Initial peers can be set in an environment variable
      peers: window.STENCILA_PEERS ? window.STENCILA_PEERS.split(' ') : [],
      // Peer discovery defaults to false but its frequency (in seconds) can be set in an environment variable
      discover: window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false,
    }),
    backend: new MemoryBackend(window.vfs),
    publicationId: getQueryStringParam('publicationId') || 'reproducible-publication'
  }, window.document.body)

})
