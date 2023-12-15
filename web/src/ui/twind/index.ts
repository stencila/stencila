import { tw } from '@twind/core'
import { LitElement } from 'lit'

import { withTwind } from '../../twind'

/**
 * This class extends LitElement to include a tw instance.
 */
@withTwind()
export class TWLitElement extends LitElement {
  protected tw: typeof tw
}
