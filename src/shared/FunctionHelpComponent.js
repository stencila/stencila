import { Component } from 'substance'

// TODO: Replace fake tutorials with real documents
const tutorials = [
  {title: 'Five-Minute introduction', link: '/tutorials.html?archive=1'},
  {title: 'Introduction to Stencila Articles', link: '/tutorials.html?archive=2'},
  {title: 'Introduction to Stencila Sheets', link: '/tutorials.html?archive=3'},
  {title: 'Polyglot Programming', link: '/tutorials.html?archive=4'},
  {title: 'Big Data', link: '/tutorials.html?archive=5'}
]


/*
  TODO: This code is really in a bad shape. We should implement this properly soon.
*/

export default class FunctionHelpComponent extends Component {
  render($$) {
    const functionManager = this.context.host.functionManager
    const func = functionManager.getFunction(this.props.functionName)

    let el = $$('div').addClass('sc-function-help')

    if (func) {
      el.append(
        $$('div').addClass('se-name').append(func.name),
        $$('div').addClass('se-description').append(func.description)
      )

      // TODO: Currently only using the first method, allow for 
      // multiple methods (ie. overloads with different parameter types)
      let method = Object.values(func.methods)[0]
      let params = method.params
      
      let syntaxEl = $$('div').addClass('se-syntax').append(
        $$('span').addClass('se-name').append(func.name),
        '('
      )
      if (params) {
        params.forEach((param, i) => {
          let paramEl = $$('span').addClass('se-signature-param').append(param.name)

          syntaxEl.append(paramEl);
          if (i < params.length - 1) {
            syntaxEl.append(',')
          }
        })
      }
      syntaxEl.append(')')

      el.append(
        $$('div').addClass('se-section-title').append('Signature'),
        syntaxEl
      )

      if (params) {
        params.forEach(param => {
          el.append(
            $$('div').addClass('se-param').append(
              $$('span').addClass('se-name').append(param.name),
              ' - ',
              $$('span').addClass('se-description').append(param.description)
            )
          )
        })
      }

      // Examples

      if(method.examples && method.examples.length > 0) {
        el.append(
          $$('div').addClass('se-section-title').append('Examples')
        )

        method.examples.forEach(example => {
          el.append(
            // $$('div').addClass('se-example-caption').append(example.caption),
            $$('pre').addClass('se-example-usage').append(example.usage)
          )
        })
      }

      el.append(
        $$('div').addClass('se-function-index').append(
          $$('a').attr({href: '#'}).append('← Function Index')
            .on('click', this._openFunctionHelp.bind(this, 'index'))
        )
      )

    } else {

      /*
      TODO: Write tutorials. Until they are done, not including these dead links

      const tutorialListEl = tutorials.map(t => $$('li').addClass('se-item').append(
        $$('a').attr('href',t.link).append(t.title)
      ))
      let tutorialsSection = $$('div').addClass('se-tutorials').append(
        $$('div').addClass('se-title').append('Getting started with Stencila'),
        $$('div').addClass('se-subtitle').append('Please read the following tutorials'),
        $$('div').addClass('se-tutorials-list').append(tutorialListEl)
      )
      */

      const functionList = functionManager.getFunctionNames()
      const functionListEl = functionList.map(func => $$('div').addClass('se-item').append(
        $$('a').attr({href: '#'})
          .append(func)
          .on('click', this._openFunctionHelp.bind(this, func))
      ))
      let functionsSection = $$('div').addClass('se-functions').append(
        $$('div').addClass('se-title').append('Functions'),
        $$('div').addClass('se-subtitle').append('Use these built-in functions'),
        $$('div').addClass('se-functions-list').append(functionListEl)
      )

      el.append(
        // tutorialsSection,
        functionsSection
      )
    }
    return el
  }

  _openFunctionHelp(funcName) {
    this.send('openHelp', `function/${funcName}`)
  }
}
