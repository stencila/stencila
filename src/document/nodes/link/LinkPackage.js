import { LinkPackage } from 'substance'
import LinkMacro from './LinkMacro'

export default {
  name: 'link',
  configure: function (config) {
    config.import(LinkPackage)
    config.addMacro(new LinkMacro())
  }
}
