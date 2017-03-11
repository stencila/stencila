import { ListPackage } from 'substance'

export default {
  name: 'list',
  configure: function (config) {
    config.import(ListPackage, {
      toolGroup: 'annotations',
      disableCollapsedCursor: true
    })
  }
}
