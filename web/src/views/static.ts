import { customElement } from 'lit/decorators.js'

import { DocumentView } from './document'

import '../nodes/all'
import '../shoelace'

/**
 * Static view of a document
 *
 * Loads components for all node types but these are all uneditable and
 * do not receive updates from a server.
 */
@customElement('stencila-static-view')
export class StaticView extends DocumentView {}
