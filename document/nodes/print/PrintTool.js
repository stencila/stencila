'use strict'

import AnnotationTool from 'substance/ui/AnnotationTool'

/**
 * A tool for editing `Print` nodes
 *
 * Updates the node's `source` property on the `change` event so that
 * errors don't get generated for incomplete input
 *
 * @class      PrintTool (name)
 */
function PrintTool () {
  PrintTool.super.apply(this, arguments)
}

PrintTool.Prototype = function () {
  var _super = PrintTool.super.prototype

  this.render = function ($$) {
    var node = this.props.node
    return _super.render.call(this, $$)
      .addClass('sc-print-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .ref('source')
              .addClass('se-source')
              .attr({
                placeholder: 'Host language expression',
                title: 'Expression to print'
              })
              .val(node ? node.source : null)
              .on('change', function (event) {
                var session = this.context.documentSession
                session.transaction(function (tx) {
                  tx.set([node.id, 'source'], event.target.value)
                })
                node.refresh()
              }.bind(this))
          )
      )
  }
}

AnnotationTool.extend(PrintTool)

export default PrintTool
