import { ToggleTool } from 'substance'
import FunctionUsageComponent from '../shared/FunctionUsageComponent'

export default class FunctionUsageTool extends ToggleTool {
  render($$) {
    let functionManager = this.context.functionManager
    let functionName = this.props.commandState.functionName
    let func = functionManager.getFunction(functionName)
    let el = $$('div').addClass('sc-function-usage-tool')
    if (func) {
      el.append(
        $$(FunctionUsageComponent, {
          spec: func,
          paramIndex: this.props.commandState.paramIndex
        })
      )
    }
    return el
  }
}
