/**
 * Browser bundle for user mode `Read`
 *
 * Only includes web components necessary for reading a static page
 * of the document.
 */

import { Mode, elevateMode } from '../mode'
elevateMode(Mode.Read)

import 'construct-style-sheets-polyfill'

export { default as StencilaDocumentHeader } from '../components/document/document-header'
//export { default as StencilaDocumentFooter } from '../components/document/document-footer'
//export { default as StencilaDocumentNav } from '../components/document/document-nav'
//export { default as StencilaDocumentToc } from '../components/document/document-toc'
//export { default as StencilaDocumentFlow } from '../components/document/document-flow'
