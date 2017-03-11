import { EmphasisPackage } from 'substance'

import EmphasisMacro from './EmphasisMacro'

export default {
  name: 'emphasis',
  configure: function (config) {
    config.import(EmphasisPackage, {disableCollapsedCursor: true})
    config.addMacro(new EmphasisMacro())
  }
}
