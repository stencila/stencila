/*
  A tiny integration of a Stencila Publication editor
  using a stub backend.
*/

import { Publication, Host, MemoryBackend, getQueryStringParam } from 'stencila'

window.addEventListener('load', () => {

  window.pub = Publication.mount({
    host: new Host({
      // Initial peers can be set in an environment variable
      peers: window.STENCILA_PEERS ? window.STENCILA_PEERS.split(' ') : [],
      // Peer discovery defaults to false but its frequency (in seconds) can be set in an environment variable
      discover: window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false,
    }),
    backend: new MemoryBackend(window.vfs),
    publicationId: getQueryStringParam('publicationId') || 'reproducible-publication'
  }, window.document.body)

})
