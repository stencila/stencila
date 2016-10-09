import Tool from 'substance/packages/tools/Tool'

/**
 * A tool for editing `Emoji` nodes
 *
 * Updates the node `name` property on the `input` event to allow for live updating.
 *
 * @class      EmojiTool (name)
 */
class EmojiTool extends Tool {

  render ($$) {
    var node = this.props.node
    return super.render.call(this, $$)
      .addClass('sc-emoji-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .ref('name')
              .addClass('se-name')
              .attr({
                placeholder: 'Emoji name',
                title: 'Name of emoji'
              })
              .val(node ? node.name : null)
              .on('input', function (event) {
                var session = this.context.documentSession
                session.transaction(function (tx) {
                  tx.set([node.id, 'name'], event.target.value)
                })
              }.bind(this))
          )
      )
  }

  shouldRerender (props) {
    // Do not re-render if the node has not changed.
    // This prevents the input box being updated during live editing
    return (this.props.node === null) || (props.node !== this.props.node)
  }
}

export default EmojiTool
