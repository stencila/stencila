import { CitationMode, CompilationMessage } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Citation` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/citation.md
 */
@customElement('stencila-citation')
@withTwind()
export class Citation extends Entity {
  @property({ attribute: 'target' })
  target?: string

  @property({ attribute: 'citation-mode' })
  citationMode?: CitationMode

  @property({ attribute: 'citation-prefix' })
  citationPrefix?: string

  @property({ attribute: 'citation-suffix' })
  citationSuffix?: string

  @property({ attribute: 'compilation-messages', type: Array })
  compilationMessages?: CompilationMessage[]

  override render() {
    return html`<slot name="content"></slot>`
  }
}
