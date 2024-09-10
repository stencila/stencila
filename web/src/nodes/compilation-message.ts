import { MessageLevel } from '@stencila/types'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `CompilationMessage` node
 *
 * Currently, this does not override `render` as it is
 * only used to display errors in CodeMirror.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-message.md
 */
@customElement('stencila-compilation-message')
@withTwind()
export class CompilationMessage extends Entity {
  @property()
  level: MessageLevel

  @property()
  message: string

  @property({ attribute: 'error-type' })
  errorType?: string

  @property({ attribute: 'code-location', type: Array })
  codeLocation?: [number, number, number, number]
}
