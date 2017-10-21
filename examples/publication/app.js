/*
  A tiny integration of a Stencila Publication editor
  using a stub backend.
*/

import { Publication, Host, MemoryBackend, getQueryStringParam, FunctionManager } from 'stencila'

window.addEventListener('load', () => {

  let peers = (getQueryStringParam('peers') || window.STENCILA_PEERS)
  if (peers) peers = peers.split(',')

  const discover = window.STENCILA_DISCOVER ? parseFloat(window.STENCILA_DISCOVER) : false

  let functionManager = new FunctionManager()
  functionManager.importLibrary('core', window.STENCILA_LIBCORE)

  let host = new Host({
    functionManager,
    peers: peers,
    discover: discover,
  })
  host.initialize().then(() => {

    let pub = Publication.mount({
      host,
      backend: new MemoryBackend(window.vfs),
      publicationId: getQueryStringParam('publicationId') || 'reproducible-publication'
    }, window.document.body)

    window.stencila = { host, pub }

  })

})
