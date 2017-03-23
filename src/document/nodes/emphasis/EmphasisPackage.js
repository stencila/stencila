import { EmphasisPackage } from 'substance'

export default {
  name: 'emphasis',
  configure: function (config) {
    config.import(EmphasisPackage, {disableCollapsedCursor: true})
  }
}
