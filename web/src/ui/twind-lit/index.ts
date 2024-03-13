import { Twind, BaseTheme } from '@twind/core'
import { LitElement } from 'lit'

import { withTwind } from '../../twind'

type Theme = BaseTheme & {
  extend?: Partial<BaseTheme>
}

/**
 * This class extends LitElement to include a tw instance.
 */
@withTwind()
export class TWLitElement extends LitElement {
  protected tw: Twind<Theme, unknown>
}
