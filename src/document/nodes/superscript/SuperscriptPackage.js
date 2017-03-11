import { SuperscriptPackage } from 'substance'

export default {
  name: 'superscript',
  configure: function (config) {
    config.import(SuperscriptPackage, { disableCollapsedCursor: true })
  }
}
