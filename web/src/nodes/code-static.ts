import { property } from 'lit/decorators.js'

import { Entity } from './entity'

/**
 * Abstract base class for web components representing Stencila Schema `CodeStatic` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md
 */
export abstract class CodeStatic extends Entity {
  @property()
  code: string

  @property({ attribute: 'code-authorship' })
  codeAuthorship?: string

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string
}
