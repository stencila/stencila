/*
  WIP: a tiny integration of a Stencila Dashboard component
  using a set of stub services.
*/

import { BackendStub, Dashboard } from 'stencila'
let stubBackend = new BackendStub()

window.addEventListener('load', () => {
  Dashboard.mount({
    backend: stubBackend
  }, window.document.body)
})
