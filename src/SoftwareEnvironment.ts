import { type, property } from './decorators'
import SoftwareApplication from './SoftwareApplication'

@type('stencila:SoftwareEnvironment')
export default class SoftwareEnvironment extends SoftwareApplication {}
