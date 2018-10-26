import { Date, Number, Text, URL } from './dataTypes'
import ComputerLanguage from './ComputerLanguage'
import CreativeWork from './CreativeWork'
import Organization from './Organization'
import Person from './Person'
import SoftwareApplication from './SoftwareApplication'

/**
 * Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
 * https://schema.org/SoftwareSourceCode
 */
export default class SoftwareSourceCode extends CreativeWork {
  codeRepository: URL = ''

  codeSampleType: Text = ''

  // codemeta:SoftwareSourceCode.maintainer
  maintainers: Array<Organization | Person> = []

  programmingLanguages: Array<ComputerLanguage> = []

  runtimePlatform: Text = ''

  targetProducts: Array<SoftwareApplication> = []
}
