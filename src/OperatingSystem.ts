import { type } from './decorators'
import Intangible from './Intangible'

@type('schema:OperatingSystem')
export default class OperatingSystem extends Intangible {

  // Instances of Operating Systems (high level)
  static linux: OperatingSystem = new OperatingSystem({ name: 'Linux' })
  static macos: OperatingSystem = new OperatingSystem({ name: 'macOS' })
  static unix: OperatingSystem = new OperatingSystem({ name: 'Unix' })
  static windows: OperatingSystem = new OperatingSystem({ name: 'Windows' })
}
