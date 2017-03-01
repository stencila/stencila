import { HeadingPackage } from 'substance'
import HeadingMacro from './HeadingMacro'

export default {
  name: 'heading',
  configure: function (config) {
    config.import(HeadingPackage)
    config.addMacro(new HeadingMacro())
  }
}
