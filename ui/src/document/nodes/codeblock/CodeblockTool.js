import BlockTool from '../../ui/BlockTool'

/**
 * A tool to edit Codeblocks
 *
 * Just changes `language` (`source` is edited via ACE editor)
 *
 * @class      CodeblockTool (name)
 */
class CodeblockTool extends BlockTool {

  render ($$) {
    var node = this.props.node
    return super.render.call(this, $$)
      .addClass('sc-codeblock-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .ref('language')
              .attr({
                placeholder: 'Enter the code language',
                spellcheck: 'false'
              })
              .val(node.language)
              .on('change', function (event) {
                var session = this.context.documentSession
                session.transaction(function (tx, args) {
                  tx.set([node.id, 'language'], event.target.value)
                })
              }.bind(this))
          )
      )
  }
}

export default CodeblockTool
