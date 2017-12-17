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

    let el = $$('div').addClass('sc-function-reference')

    if(functionInstance) {
      const usage = functionInstance.getUsage()
      el.append(
        $$('div').addClass('se-name').append(usage.name),
        $$('div').addClass('se-description').append(usage.summary)
      )

      if(usage.examples.length > 0) {
        el.append(
          $$('div').addClass('se-section-title').append(this.getLabel('function-examples'))
        )

        usage.examples.forEach(example => {
          el.append(
            $$('div').addClass('se-example').append(example)
          )
        })
      }

      let syntaxEl = $$('div').addClass('se-syntax').append(
        $$('span').addClass('se-name').append(usage.name),
        '('
      )

      usage.params.forEach((param, i) => {
        let paramEl = $$('span').addClass('se-signature-param').append(param.name)

        syntaxEl.append(paramEl);
        if (i < usage.params.length - 1) {
          syntaxEl.append(',')
        }
      })

      syntaxEl.append(')')

      el.append(
        $$('div').addClass('se-section-title').append(this.getLabel('function-usage')),
        syntaxEl
      )

      usage.params.forEach(param => {
        el.append(
          $$('div').addClass('se-param').append(
            $$('span').addClass('se-name').append(param.name),
            ' - ',
            $$('span').addClass('se-description').append(param.description)
          )
        )
      })

    } else {
      const functionList = functionManager.getFunctionNames()
      functionList.forEach(func => {
        el.append(
          $$('div').addClass('se-item').append(func)
        )
      })
    }

    return el
  }


}
