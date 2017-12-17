import { Component } from 'substance'

export default class FunctionReferenceComponent extends Component {

  didMount() {

  }

  dispose() {

  }

  render($$) {
    const sectionId = this.props.sectionId
    const functionManager = this.context.host.functionManager
    const functionInstance = functionManager.getFunction(sectionId)

    let el = $$('div').addClass('sc-function-reference').append(sectionId)

    if(functionInstance) {
      const functionUsage = functionInstance.getUsage()
    } else {
      const functionList = functionManager.getFunctionNames()
    }

    return el
  }


}
