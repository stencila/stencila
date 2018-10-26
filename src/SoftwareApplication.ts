import CreativeWork from './CreativeWork'
import SoftwarePackage from './SoftwarePackage'

/**
 * A software application.
 * https://schema.org/SoftwareApplication
 */
export default class SoftwareApplication extends CreativeWork {

  /**
   * The [`schema:softwareRequirements`](https://schema.org/softwareRequirements)
   * property allows for `Text` or `URL` values. Here, we allow
   * values of software packages or applications.
   */
  softwareRequirements: Array<SoftwarePackage | SoftwareApplication> = []

}
