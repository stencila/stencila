import SoftwareApplication from './SoftwareApplication'
import SoftwareSourceCode from './SoftwareSourceCode'

/**
 * An extension class defined for this context to represent a software
 * package. We considered this necessary because `schema:SoftwareSourceCode`
 * has most properties needed to represent a package but not all of them.
 * Meanwhile, `schema:SoftwareApplication` has some of those missing
 * properties but lacks most of those needed. Thus, this type does
 * not introduce any new properties, but rather uses
 * schema.org properties on a subtype of `schema:SoftwareSourceCode`
 */
export default class SoftwarePackage extends SoftwareSourceCode {

  /**
   * The [`schema:softwareRequirements`](https://schema.org/softwareRequirements)
   * property allows for `Text` or `URL` values. Here, we allow
   * values of software packages or applications.
   */
  softwareRequirements: Array<SoftwarePackage | SoftwareApplication> = []

}
