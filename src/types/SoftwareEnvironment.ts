import { type } from './decorators'
import SoftwareApplication from './SoftwareApplication'

/**
 * A software environment
 *
 * Currently only used in [Dockter](https://github.com/stencila/dockter)
 * form which a Dockerfile is generated.
 *
 * This may be replaced by `openschemas:Container`.
 * See https://github.com/stencila/schema/issues/11
 */
@type('stencila:SoftwareEnvironment')
export default class SoftwareEnvironment extends SoftwareApplication {}
