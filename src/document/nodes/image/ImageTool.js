import BlockTool from '../../ui/BlockTool'

/**
 * A tool to edit images
 *
 * @class      ImageTool (name)
 */
class ImageTool extends BlockTool {

  render ($$) {
    // For placeholder to work override Substance's
    // default for src
    var src = this.props.node.src
    if (src === 'http://') src = ''
    return super.render.call(this, $$)
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

  onChange (event) {
    var node = this.props.node
    var session = this.context.documentSession
    session.transaction(function (tx) {
      tx.set([node.id, 'src'], event.target.value)
    })
  }
}

export default ImageTool
