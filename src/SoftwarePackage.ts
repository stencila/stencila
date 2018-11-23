import { type, property } from './decorators'
import OperatingSystem from './OperatingSystem'
import SoftwareApplication from './SoftwareApplication'
import SoftwareSourceCode from './SoftwareSourceCode'

/**
 * A software package.
 *
 * This is an extension class defined for this context.
 * It is necessary because `schema:SoftwareSourceCode`
 * has most, but not all, of the properties that we need to represent a package,
 * for applications such as Dockter.
 * Meanwhile, `schema:SoftwareApplication` has some of those missing
 * properties but lacks most. This type does
 * not introduce any new properties, but rather uses
 * schema.org properties on a subtype of `schema:SoftwareSourceCode`
 *
 * An alternative approach would be to create a `SoftwareApplication` which
 * links to one or more `SoftwarePackages`. See https://github.com/codemeta/codemeta/issues/198
 */
@type('stencila:SoftwarePackage')
export default class SoftwarePackage extends SoftwareSourceCode {

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
