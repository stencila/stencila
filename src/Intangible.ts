import { type } from './decorators'
import Thing from './Thing'

/**
 * A utility class that serves as the umbrella for a number of 'intangible'
 * things such as quantities, structured values, etc.
 * 
 * @see {@link https://schema.org/Intangible}
 */
@type('schema:Intangible')
export default class Intangible extends Thing {}
