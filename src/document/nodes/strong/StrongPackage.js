import { StrongPackage } from 'substance'

export default {
  name: 'strong',
  configure: function (config) {
    config.import(StrongPackage, { disableCollapsedCursor: true })
  }
}
