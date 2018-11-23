import { type } from './decorators'
import Intangible from './Intangible'

/**
 * This type covers computer programming languages such as Scheme and Lisp,
 * as well as other language-like computer representations.
 * Natural languages are best represented with the Language type.
 * 
 * @see {@link https://schema.org/ComputerLanguage}
 */
@type('schema:ComputerLanguage')
export default class ComputerLanguage extends Intangible {
  // Instances of computer languages
  static js: ComputerLanguage = new ComputerLanguage({ name: 'JavaScript' })
  static py: ComputerLanguage = new ComputerLanguage({ name: 'Python' })
  static r: ComputerLanguage = new ComputerLanguage({ name: 'R' })
}
