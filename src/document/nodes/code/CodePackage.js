import { CodePackage } from 'substance'

export default {
  name: 'code',
  configure: function (config) {
    config.import(CodePackage, { disableCollapsedCursor: true })
  }
}
