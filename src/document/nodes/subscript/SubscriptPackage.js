import { SubscriptPackage } from 'substance'

export default {
  name: 'subscript',
  configure: function (config) {
    config.import(SubscriptPackage, { disableCollapsedCursor: true })
  }
}
