import Intangible from './Intangible'

export default class ComputerLanguage extends Intangible {

  // Instances of
  static js: ComputerLanguage = new ComputerLanguage({ name: 'JavaScript' })
  static py: ComputerLanguage = new ComputerLanguage({ name: 'Python' })
  static r: ComputerLanguage = new ComputerLanguage({ name: 'R' })
}
