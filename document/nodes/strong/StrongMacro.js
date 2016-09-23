import AnnotationMacro from '../../ui/AnnotationMacro'

/**
 * A macro for creating `Strong` nodes
 *
 * Uses enclosing asterisk.
 *
 * Note that this is different to Markdown which uses double asterisk or double underscores
 * for "strong emphasis" (here, strong means "strong importance" https://developer.mozilla.org/en/docs/Web/HTML/Element/strong#Emphasis_vs._Strong)
 *
 * @class      StrongMacro (name)
 */
function StrongMacro () {
};

StrongMacro.Prototype = function () {
  this.appliesTo = []

  this.regex = /\*([^\*]+)\*/

  this.createNodeData = function (match) {
    return {
      type: 'strong',
      text: match[1]
    }
  }
}

AnnotationMacro.extend(StrongMacro)

export default StrongMacro
