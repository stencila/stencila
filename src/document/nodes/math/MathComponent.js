import { Component } from 'substance'

import math from '../../../utilities/math/index'

class MathComponent extends Component {

  didMount () {
    this.props.node.on('source:changed', this.rerender, this)
    this.props.node.on('language:changed', this.rerender, this)
    this.props.node.on('display:changed', this.rerender, this)
  }

  dispose () {
    this.props.node.off(this)
  }

  render ($$) {
    var node = this.props.node

    var el = $$('span')
      .addClass('sc-math sm-' + node.language)
      .ref('math')

    try {
      el.html(
        math.render(node.source, node.language, node.display)
      )
    } catch (error) {
      el.addClass('sm-error')
        .text(error.message)
    }

    if (node.display === 'block') {
      el.addClass('sm-block')
    }

    return el
  }

}

export default MathComponent
