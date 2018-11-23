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

  /**
   * Linux operating system family that use the Linux kernel. For instance GNU/Linux or Android.
   *
   * @see {@link https://www.wikidata.org/wiki/Q388}
   */
  static linux: OperatingSystem = new OperatingSystem({ name: 'Linux' })

  /**
   * macOS operating system for Apple computers, launched in 2001 as Mac OS X
   *
   * @see {@link https://www.wikidata.org/wiki/Q14116}
   */
  static macos: OperatingSystem = new OperatingSystem({ name: 'macOS' })

  /**
   * Unix family of computer operating systems that derive from the original AT&T Unix
   *
   * @see {@link https://www.wikidata.org/wiki/Q11368}
   */
  static unix: OperatingSystem = new OperatingSystem({ name: 'Unix' })

  /**
   * Windows family of operating systems produced for personal computers,
   * servers, smartphones and embedded devices
   *
   * @see {@link https://www.wikidata.org/wiki/Q1406}
   */
  static windows: OperatingSystem = new OperatingSystem({ name: 'Windows' })
}
