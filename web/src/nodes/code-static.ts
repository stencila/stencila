import { property } from 'lit/decorators.js'

import { Entity } from './entity'

/**
 * Abstract base class for web components representing Stencila Schema `CodeStatic` node types
 *
 * Note that `code` and `authors` are encoded as slots.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md
 */
export abstract class CodeStatic extends Entity {
  @property({ attribute: 'programming-language' })
  programmingLanguage?: string
}
