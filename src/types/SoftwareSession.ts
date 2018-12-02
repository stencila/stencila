import { type } from './decorators'
import Thing from './Thing'

/**
 * Represents a session within a `SoftwareEnvironment`
 *
 * We may be able to use `openschemas::Container` instead.
 */
@type('stencila:SoftwareSession')
export default class SoftwareSession extends Thing {}
