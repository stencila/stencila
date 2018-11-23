import { type, property } from './decorators'
import { Date, Number, Text, URL } from './dataTypes'
import ComputerLanguage from './ComputerLanguage'
import CreativeWork from './CreativeWork'
import Organization from './Organization'
import Person from './Person'
import SoftwareApplication from './SoftwareApplication'

/**
 * Computer programming source code. Example: Full (compile ready) solutions,
 * code snippet samples, scripts, templates.
 * 
 * @see {@link https://schema.org/SoftwareSourceCode}
 */
@type('schema:SoftwareSourceCode')
export default class SoftwareSourceCode extends CreativeWork {

  /**
   * Link to the repository where the un-compiled, human readable code and
   * related code is located (SVN, github, CodePlex).
   * 
   * @see {@link https://schema.org/codeRepository}
   */
  @property('schema:codeRepository')
  codeRepository: URL = ''

  /**
   * What type of code sample: full (compile ready) solution, code snippet,
   * inline code, scripts, template.
   * 
   * @see {@link https://schema.org/codeSampleType}
   */
  @property('schema:codeSampleType')
  codeSampleType: Text = ''

  /**
   * Individual responsible for maintaining the software (usually includes an email contact address).
   * 
   * Note that CodeMeta says that `maintainer` should be a `Person`, not `Organization` or `Person`
   * as with `author`
   * 
   * @see {@link https://codemeta.github.io/terms/}
   */
  @property('codemeta:maintainer')
  maintainers: Array<Person> = []

  /**
   * The computer programming language.
   * 
   * @see {@link https://schema.org/programmingLanguage}
   */
  @property('schema:programmingLanguage')
  programmingLanguages: Array<ComputerLanguage | Text> = []

  /**
   * Runtime platform or script interpreter dependencies (Example - Java v1, 
   * Python2.3, .Net Framework 3.0).
   * 
   * @see {@link https://schema.org/runtimePlatform}
   */
  @property('schema:runtimePlatform')
  runtimePlatform: Text = ''

  /**
   * Target Operating System / Product to which the code applies. If applies to
   * several versions, just the product name can be used.
   * 
   * @see {@link https://schema.org/targetProduct}
   */
  @property('schema:targetProduct')
  targetProducts: Array<SoftwareApplication> = []
}
