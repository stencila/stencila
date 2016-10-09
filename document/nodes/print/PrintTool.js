import AnnotationTool from 'substance/ui/AnnotationTool'

/**
 * A tool for editing `Print` nodes
 *
 * Updates the node's `source` property on the `change` event so that
 * errors don't get generated for incomplete input
 *
 * @class      PrintTool (name)
 */
class PrintTool extends AnnotationTool {

  render ($$) {
    var node = this.props.node
    return super.render.call(this, $$)
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

export default PrintTool
