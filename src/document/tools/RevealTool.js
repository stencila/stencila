import Tool from 'substance/packages/tools/Tool'

/**
 * Tool for toggling the reveal mode of a
 * Stencila Document `VisualEditor`
 *
 * @class      RevealTool (name)
 */
class RevealTool extends Tool {

  getTitle () {
    if (this.props.active) return 'Don\'t show computations and comments'
    else return 'Show computations and comments'
  }

  onClick () {
    this.send('reveal-toggle')
  }

}

export default RevealTool

