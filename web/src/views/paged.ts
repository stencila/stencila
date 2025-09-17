import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import 'pagedjs/dist/paged.polyfill'

/**
 * Paged view of a document intended mainly for use when generating PDFs in the
 * browser
 *
 * Loads Paged.js (https://github.com/pagedjs/pagedjs) to support like CSS Paged
 * Media Rules like @page, @top-left, and @bottom-center.
 *
 * Does NOT extend DocumentView because no document menu is needed/wanted in
 * these contexts.
 *
 * Does NOT load any custom web components for Stencila nodes as these do not
 * play well with the Paged.js polyfill (causes exceptions).
 *
 * Uses light DOM (this is render root) so that theme styles are applied to the
 * document content.
 */
@customElement('stencila-paged-view')
export class PagedView extends LitElement {
  override createRenderRoot() {
    return this
  }
}
