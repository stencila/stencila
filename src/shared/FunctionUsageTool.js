import { ToggleTool } from 'substance'
import FunctionUsageComponent from '../shared/FunctionUsageComponent'

export default class FunctionUsageTool extends ToggleTool {
  render($$) {
    let functionManager = this.context.functionManager
    let func = functionManager.getFunction(this.props.commandState.functionName)
    let el = $$('div').addClass('sc-function-usage-tool')
    if (func) {
      el.append(
        $$(FunctionUsageComponent, {
          spec: func.getUsage(),
          paramIndex: this.props.commandState.paramIndex
        })
      )
    }
    return el
  }
}
