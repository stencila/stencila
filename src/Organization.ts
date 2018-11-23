import { type } from './decorators'
import Thing from './Thing'

/**
 * An organization such as a school, NGO, corporation, club, etc.
 *
 * @see {@link https://schema.org/Organization}
 */
@type('schema:Organization')
export default class Organization extends Thing {}
