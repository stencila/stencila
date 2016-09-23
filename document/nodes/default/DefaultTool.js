'use strict'

import BlockTool from '../../ui/BlockTool'

/**
 * A tool to edit `Default` nodes
 *
 * @class      DefaultTool (name)
 */
function DefaultTool () {
  DefaultTool.super.apply(this, arguments)
}

DefaultTool.Prototype = function () {
  var _super = DefaultTool.super.prototype

  this.render = function ($$) {
    var node = this.props.node
    return _super.render.call(this, $$)
      .addClass('sc-default-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('button')
              .ref('edit')
              .addClass('se-edit')
              .attr('title', 'Edit')
              .append(
                $$('i')
                  .addClass('fa fa-pencil')
              )
              .on('click', function (event) {
                node.emit('edit:toggle')
              })
          )
      )
  }
}

BlockTool.extend(DefaultTool)

export default DefaultTool
