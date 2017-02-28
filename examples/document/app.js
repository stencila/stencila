/*
  WIP: a tiny integration of a Stencila Document editor
  using a set of stub services.
*/

import wrapSnippet from '../docs/wrapSnippet'
import example from '../docs/kitchensink'
import { DocumentApp } from 'stencila-document'

window.addEventListener('load', () => {
  DocumentApp.mount({
    edit: true,
    format: 'html',
    data: wrapSnippet(example)
  }, window.document.body)
})
