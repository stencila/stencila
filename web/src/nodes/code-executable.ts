import { property } from 'lit/decorators.js'

import { AuthorshipRun } from '../ui/nodes/properties/code/types'

import { Executable } from './executable'

/**
 * Abstract base class for web components representing Stencila Schema `CodeExecutable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-executable.md
 */
export abstract class CodeExecutable extends Executable {
  @property()
  code: string

  @property({ attribute: 'code-authorship', type: Array })
  codeAuthorship?: AuthorshipRun[]

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string
}
