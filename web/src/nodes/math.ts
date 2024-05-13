import { property } from 'lit/decorators'

import { Entity } from './entity'

/**
 * Abstract base class for components representing Stencila Schema `Math` types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md
 */
export abstract class Math extends Entity {
  @property({ type: String, attribute: 'math-language' })
  mathLanguage: string = 'tex'

  @property({ type: String })
  code: string
}
