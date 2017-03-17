import VegaLiteComponent from './VegaLiteComponent'

export default {
  name: 'vega-lite',
  configure: function (config) {
    config.addComponent('value:vegalite', VegaLiteComponent)
  }
}
