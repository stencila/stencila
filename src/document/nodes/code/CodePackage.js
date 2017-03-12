import { CodePackage } from 'substance'
import CodeMacro from './CodeMacro'

export default {
  name: 'code',
  configure: function (config) {
    config.import(CodePackage, { disableCollapsedCursor: true })
    config.addMacro(new CodeMacro())
  }
}
