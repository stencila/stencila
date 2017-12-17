import { Component } from 'substance'


export default class FunctionUsageComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-function-usage')
    let spec = this.props.spec
    let paramIndex = this.props.paramIndex

    // Function signature
    let signatureEl = $$(FunctionSignature, {
      spec,
      paramIndex
    })

    // Parameter descriptions
    let paramsEl = $$('table').addClass('se-parameters')
    spec.params.forEach((param, i) => {
      let paramEl = $$('tr').addClass('se-param').append(
        $$('td').addClass('se-param-name').append(param.name),
        $$('td').addClass('se-param-descr').append(param.description)
      )
      if (i === this.props.paramIndex) {
        paramEl.addClass('sm-active')
      }
      paramsEl.append(paramEl)
    })

    // let summaryEl = $$('div').addClass('se-summary').append(spec.summary)

    let documentationLink = $$('div').addClass('se-read-more').append(
      this.context.iconProvider.renderIcon($$, 'function-helper')
    ).on('click', this._openDocumentation)

    // Documentation
    let docEl = $$('div').addClass('se-documentation')
    docEl.append(
      signatureEl,
      documentationLink
      // paramsEl,
      // summaryEl
    )
    el.append(docEl)
    //
    // if (spec.examples.length > 0) {
    //   // Display first example
    //   let example = spec.examples[0]
    //   el.append(
    //     $$('div').addClass('se-example').append(
    //       $$('div').addClass('se-label').append('Example'),
    //       $$('div').addClass('se-example-code').append(example)
    //     )
    //   )
    // }
    return el
  }

  _openDocumentation() {
    const spec = this.props.spec
    this.send('help', 'function-reference:' + spec.name)
  }
}

class FunctionSignature extends Component {
  render($$) {
    let spec = this.props.spec
    let paramsEl = $$('span').addClass('se-signature-params');

    spec.params.forEach((param, i) => {
      let paramEl = $$('span').addClass('se-signature-param').append(param.name)

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
