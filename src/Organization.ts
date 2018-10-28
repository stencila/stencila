import { type } from './decorators'
import Thing from './Thing'

@type('schema:Organization')
export default class Organization extends Thing {}
