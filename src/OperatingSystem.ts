import { type } from './decorators'
import Intangible from './Intangible'

/**
 * A collection of software that manages computer hardware resources
 * 
 * @see {@link https://www.wikidata.org/wiki/Q9135}
 */
@type('stencila:OperatingSystem')
export default class OperatingSystem extends Intangible {

  // Instances of OperatingSystem (high level)
  static linux: OperatingSystem = new OperatingSystem({ name: 'Linux' })
  static macos: OperatingSystem = new OperatingSystem({ name: 'macOS' })
  static unix: OperatingSystem = new OperatingSystem({ name: 'Unix' })
  static windows: OperatingSystem = new OperatingSystem({ name: 'Windows' })
}
