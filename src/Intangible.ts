import { type } from './decorators'
import Thing from './Thing'

@type('schema:Intangible')
export default class Intangible extends Thing {}
