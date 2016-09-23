'use strict'

import BlockTool from '../../ui/BlockTool'

/**
 * A tool to edit images
 *
 * @class      ImageTool (name)
 */
function ImageTool () {
  ImageTool.super.apply(this, arguments)
}

ImageTool.Prototype = function () {
  var _super = ImageTool.super.prototype

  this.render = function ($$) {
    // For placeholder to work override Substance's
    // default for src
    var src = this.props.node.src
    if (src === 'http://') src = ''
    return _super.render.call(this, $$)
      .addClass('sc-image-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .attr({
                value: src,
                placeholder: 'Paste or type a URL',
                spellcheck: 'false'
              })
              .on('change', this.onChange.bind(this))
          )
      )
  }

  this.onChange = function (event) {
    var node = this.props.node
    var session = this.context.documentSession
    session.transaction(function (tx) {
      tx.set([node.id, 'src'], event.target.value)
    })
  }
}

BlockTool.extend(ImageTool)

module.exports = ImageTool
