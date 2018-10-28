import { type } from './decorators'
import Intangible from './Intangible'

@type('schema:ComputerLanguage')
export default class ComputerLanguage extends Intangible {

  // Instances of computer languages
  static js: ComputerLanguage = new ComputerLanguage({ name: 'JavaScript' })
  static py: ComputerLanguage = new ComputerLanguage({ name: 'Python' })
  static r: ComputerLanguage = new ComputerLanguage({ name: 'R' })
}
