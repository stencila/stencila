/**
 * Browser bundle for user mode `Read`
 *
 * Only includes web components necessary for reading a static page
 * of the document.
 */

import { Mode, elevateMode } from '../mode'
elevateMode(Mode.Read)

import 'construct-style-sheets-polyfill'

import '../components/document/document-header'
// import '../components/document/document-footer'
// import '../components/document/document-nav'
// import '../components/document/document-toc'
// import '../components/document/document-flow'
