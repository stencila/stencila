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
  @property('schema:softwareRequirements', 'list')
  softwareRequirements: Array<SoftwarePackage | SoftwareApplication> = []

  @property('schema:applicationCategory', 'list')
  applicationCategories: Array<Text | URL> = []

  @property('schema:applicationSubCategory', 'list')
  applicationSubCategories: Array<Text | URL> = []

  @property('schema:operatingSystem', 'list')
  operatingSystems: Array<Text> = []
}
