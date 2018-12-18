import { type, property } from './decorators'
import CreativeWork from './CreativeWork'
import SoftwarePackage from './SoftwarePackage'
import OperatingSystem from './OperatingSystem'
import { Text } from './dataTypes'

/**
 * A software application.
 *
 * @see {@link https://schema.org/SoftwareApplication}
 */
@type('schema:SoftwareApplication')
export default class SoftwareApplication extends CreativeWork {

  /**
   * Type of software application, e.g. 'Game, Multimedia'.
   *
   * @see {@link https://schema.org/applicationCategory}
   */
  @property('schema:applicationCategory')
  applicationCategories: Array<Text | URL> = []

  /**
   * Subcategory of the application, e.g. 'Arcade Game'.
   *
   * @see {@link https://schema.org/applicationSubCategory}
   */
  @property('schema:applicationSubCategory')
  applicationSubCategories: Array<Text | URL> = []

  /**
   * Operating systems supported (Windows 7, OSX 10.6, Android 1.6).
   *
   * @see {@link https://schema.org/operatingSystem}
   */
  @property('schema:operatingSystem')
  operatingSystems: Array<OperatingSystem> = []

  /**
   * Component dependency requirements for application.
   * This includes runtime environments and shared libraries that are not included in
   * the application distribution package, but required to run the application.
   *
   * The [`schema:softwareRequirements`](https://schema.org/softwareRequirements)
   * property allows for `Text` or `URL` values. Here, we allow
   * values of software packages or applications.
   *
   * @see {@link https://schema.org/softwareRequirements}
   */
  @property('schema:softwareRequirements')
  softwareRequirements: Array<SoftwarePackage | SoftwareApplication> = []
}
