import AnnotationMacro from '../../ui/AnnotationMacro'

/**
 * A macro for creating `Link` nodes
 *
 * Uses Markdown syntax:
 *
 *   [text](url)
 *
 * @class      LinkMacro (name)
 */
class LinkMacro extends AnnotationMacro {

  static get appliesTo () {
    return []
  }

  static get regex () {
    return /\[([^\]]+)\]\(([^\)]+)\)/
  }

  createNodeData (match) {
    return {
      type: 'link',
      text: match[1],
      url: match[2]
    }
  }

}

export default LinkMacro
