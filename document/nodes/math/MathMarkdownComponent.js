'use strict'

import Component from 'substance/ui/Component'
import TextPropertyEditor from 'substance/ui/TextPropertyEditor'

function MathCodeComponent () {
  MathCodeComponent.super.apply(this, arguments)
}

MathCodeComponent.Prototype = function () {
  this.render = function ($$) {
    var node = this.props.node

    var delim
    if (node.language === 'asciimath') {
      delim = '|'
    } else {
      delim = '$'
    }

    return $$('span')
      .addClass('sc-math')
      .append(
        delim,
        $$(TextPropertyEditor, {
          path: [ node.id, 'source' ],
          withoutBreak: true
        }),
        delim
      )
  }
}

Component.extend(MathCodeComponent)

export default MathCodeComponent
