import { Component } from 'substance'

// TODO: Replace fake tutorials with real documents
const tutorials = [
  {title: 'Five-Minute introduction', link: '/tutorials.html?archive=1'},
  {title: 'Introduction to Stencila Articles', link: '/tutorials.html?archive=2'},
  {title: 'Introduction to Stencila Sheets', link: '/tutorials.html?archive=3'},
  {title: 'Polyglot Programming', link: '/tutorials.html?archive=4'},
  {title: 'Big Data', link: '/tutorials.html?archive=5'}
]

export default class FunctionHelpComponent extends Component {
  render($$) {
    const functionManager = this.context.host.functionManager
    const functionInstance = functionManager.getFunction(this.props.functionName)

    let el = $$('div').addClass('sc-function-help')

    if (functionInstance) {
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

      const tutorialListEl = tutorials.map(t => $$('li').addClass('se-item').append(
        $$('a').attr('href',t.link).append(t.title)
      ))
      let tutorialsSection = $$('div').addClass('se-tutorials').append(
        $$('div').addClass('se-title').append('Getting started with Stencila'),
        $$('div').addClass('se-subtitle').append('Please read the following tutorials'),
        $$('div').addClass('se-tutorials-list').append(tutorialListEl)
      )

      const functionList = functionManager.getFunctionNames()
      const functionListEl = functionList.map(func => $$('div').addClass('se-item').append(
        $$('a').attr({href: '#'})
          .append(func)
          .on('click', this._openFunctionHelp.bind(this, func))
      ))
      let functionsSection = $$('div').addClass('se-functions').append(
        $$('div').addClass('se-title').append('Functions'),
        $$('div').addClass('se-subtitle').append('Use the following built-in functions of Stencila'),
        $$('div').addClass('se-functions-list').append(functionListEl)
      )

      el.append(
        tutorialsSection,
        functionsSection
      )
    }
    return el
  }

  _openFunctionHelp(funcName) {
    this.send('openHelp', `function/${funcName}`)
  }
}
