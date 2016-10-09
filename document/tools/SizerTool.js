import Tool from 'substance/packages/tools/Tool'

/**
 * Tool for toggling the size of a toolset
 *
 * @class      SizerTool (name)
 */
class SizerTool extends Tool {

  render ($$) {
    return $$('div').attr({
      'class': 'sc-tool sc-sizer-tool',
      'title': this.props.maximized ? 'Minimize' : 'Maximize'
    }).append(
      $$('button').append(
        $$('i').addClass('fa fa-' + (this.props.maximized ? 'chevron-up' : 'circle'))
      ).on('click', () => {
        this.send('toggle-maximized')
      })
    )
  }

}

export default SizerTool
