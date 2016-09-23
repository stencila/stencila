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
function LinkMacro () {
};

LinkMacro.Prototype = function () {
  this.appliesTo = []

  this.regex = /\[([^\]]+)\]\(([^\)]+)\)/

  this.createNodeData = function (match) {
    return {
      type: 'link',
      text: match[1],
      url: match[2]
    }
  }
}

AnnotationMacro.extend(LinkMacro)

export default LinkMacro
