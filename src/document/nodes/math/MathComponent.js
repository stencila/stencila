import { Component } from 'substance'

import math from '../../../utilities/math/index'

class MathComponent extends Component {

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

    return el
  }

}

export default MathComponent
