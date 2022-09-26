import { property } from 'lit/decorators'

import StencilaElement from '../utils/element'

/**
 * A base component to represent the `Entity` node type
 */
export default class StencilaEntity extends StencilaElement {
  @property()
  id: string
}
