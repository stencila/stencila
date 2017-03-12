import { Component } from 'substance'

import math from '../../../utilities/math/index'

class MathComponent extends Component {

  render ($$) {
    var node = this.props.node

    var el = $$('span')
      .addClass('sc-math sm-' + node.language)
      .ref('math')

    if (this.props.isolatedNodeState) {
      el.addClass('sm-'+this.props.isolatedNodeState)
    }
    try {
      el.append(
        $$('span').addClass('se-rendered-math').html(
          math.render(node.source, node.language, node.display)
        )
      )
      let blockerEl = $$('div').addClass('se-blocker')
      el.append(blockerEl)
    } catch (error) {
      el.addClass('sm-error')
        .text(error.message)
    }
    return el
  }

}

export default MathComponent
