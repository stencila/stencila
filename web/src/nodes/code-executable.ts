import { property } from 'lit/decorators.js'

import { Executable } from './executable'

/**
 * Abstract base class for web components representing Stencila Schema `CodeExecutable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-executable.md
 */
export abstract class CodeExecutable extends Executable {
  @property()
  code: string

  /**
   * A 'stringified' JS array,
   * Important should be kept as a `String`
   * to avoid errors when passing down to children.
   */
  @property({ attribute: 'code-authorship' })
  codeAuthorship?: string

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string
}
