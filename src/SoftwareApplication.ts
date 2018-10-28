import { type, property } from './decorators'
import CreativeWork from './CreativeWork'
import SoftwarePackage from './SoftwarePackage'

@type('schema:SoftwareApplication')
export default class SoftwareApplication extends CreativeWork {

  /**
   * The [`schema:softwareRequirements`](https://schema.org/softwareRequirements)
   * property allows for `Text` or `URL` values. Here, we allow
   * values of software packages or applications.
   */
  @property('schema:softwareRequirements')
  softwareRequirements: Array<SoftwarePackage | SoftwareApplication> = []

}
