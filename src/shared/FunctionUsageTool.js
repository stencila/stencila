import { ToggleTool, FontAwesomeIcon } from 'substance'
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
          spec: func.getUsage(),
          paramIndex: this.props.commandState.paramIndex
        })
      )
    } else {
      el.append(
        $$('div').addClass('se-function-not-found').append(
          $$(FontAwesomeIcon, { icon: 'fa-warning' }),
          ` Function ${functionName} does not exist`
        )
      )
    }
    return el
  }
}
