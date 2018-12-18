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

  /**
   * Javascript programming language
   *
   * @see {@link https://www.wikidata.org/wiki/Q2005}
   */
  static js: ComputerLanguage = new ComputerLanguage({ name: 'JavaScript' })

  /**
   * Python general-purpose, high-level programming language
   *
   * @see {@link https://www.wikidata.org/wiki/Q28865}
   */
  static py: ComputerLanguage = new ComputerLanguage({ name: 'Python' })

  /**
   * R programming language for statistical computing
   *
   * @see {@link https://www.wikidata.org/wiki/Q206904}
   */
  static r: ComputerLanguage = new ComputerLanguage({ name: 'R' })
}
