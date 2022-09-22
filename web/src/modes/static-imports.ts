/**
 * Browser bundle for user mode `Static`
 *
 * Only includes web components necessary for reading a static page
 * of the document.
 */

import { initialize } from '../components/utils/css'
initialize()

import '../components/document/document-header'
import '../components/document/document-footer'
import '../components/document/document-nav'
import '../components/document/document-toc'
