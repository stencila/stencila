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
    const node = this.props.node
    const el = super.render($$)
    el.addClass('sc-codeblock-tool')

    const details = $$('div').ref('details').addClass('se-details')
    details.append(
      $$('input').ref('language')
        .attr({
          placeholder: 'Enter the code language',
          spellcheck: 'false'
        })
        .val(node.language)
        .on('change', this.onLanguageChange)
    )
    el.append(details)
    return el
  }

  onLanguageChange(event) {
    const node = this.props.node
    const session = this.context.editorSession
    session.transaction((tx) => {
      tx.set([node.id, 'language'], event.target.value)
    })
  }
}

export default CodeblockTool