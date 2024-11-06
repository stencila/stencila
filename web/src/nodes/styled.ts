import { property } from 'lit/decorators.js'

import { Entity } from './entity'

/**
 * Abstract base class for web components representing Stencila Schema `Styled` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/styled/styled.md
 */
export abstract class Styled extends Entity {
  @property()
  code: string

  @property({ attribute: 'code-authorship' })
  codeAuthorship?: string

  @property({ attribute: 'style-language' })
  styleLanguage?: string
}
