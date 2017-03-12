import { StrongPackage } from 'substance'
import StrongMacro from './StrongMacro'

export default {
  name: 'strong',
  configure: function (config) {
    config.import(StrongPackage, { disableCollapsedCursor: true })
    config.addMacro(new StrongMacro())
  }
}
