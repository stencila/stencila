import { Component } from 'substance'


export default class FunctionUsageComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-function-usage')
    let spec = this.props.spec
    let paramIndex = this.props.paramIndex

    // Function signature
    var signatureEl = $$(FunctionSignature, {
      spec,
      paramIndex
    })

    // Parameter descriptions
    var paramsEl = $$('table').addClass('se-parameters')
    spec.params.forEach((param, i) => {
      var paramEl = $$('tr').addClass('se-param').append(
        $$('td').addClass('se-param-name').append(param.name),
        $$('td').addClass('se-param-descr').append(param.description)
      )
      if (i === this.props.paramIndex) {
        paramEl.addClass('sm-active')
      }
      paramsEl.append(paramEl)
    })

    let summaryEl = $$('div').addClass('se-summary').append(spec.summary)

    // Documentation
    let docEl = $$('div').addClass('se-documentation')
    docEl.append(
      signatureEl,
      paramsEl,
      summaryEl
    )
    el.append(docEl)

    if (spec.examples.length > 0) {
      // Display first example
      let example = spec.examples[0]
      el.append(
        $$('div').addClass('se-example').append(
          $$('div').addClass('se-label').append('Example'),
          $$('div').addClass('se-example-code').append(example)
        )
      )
    }
    return el
  }
}


class FunctionSignature extends Component {
  render($$) {
    var spec = this.props.spec
    var paramsEl = $$('span').addClass('se-signature-params');

    spec.params.forEach((param, i) => {
      var paramEl = $$('span').addClass('se-signature-param').append(param.name)

      if (i === this.props.paramIndex) {
        paramEl.addClass('sm-active')
      }

      paramsEl.append(paramEl);
      if (i < spec.params.length - 1) {
        paramsEl.append(',')
      }
    })

    return $$('div').addClass('se-signature').append(
      $$('span').addClass('se-name').append(spec.name),
      '(',
      $$('span').append(paramsEl),
      ')'
    )
  }
}
