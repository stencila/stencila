import { Twind, BaseTheme } from '@twind/core'
import { LitElement } from 'lit'
import { config } from '../../twind'

const t = config['theme']
type Theme = BaseTheme & {
  extend?: Partial<BaseTheme>
}

/**
 * This class extends LitElement to include a tw instance.
 */
export class TWLitElement extends LitElement {
  protected tw: Twind<Theme, unknown>
}
